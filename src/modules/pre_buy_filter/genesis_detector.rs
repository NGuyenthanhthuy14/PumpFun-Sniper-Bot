/// Phase 2 — Genesis Bundle Detection (Nâng cao)
///
/// Phát hiện dev dùng Jito bundles hoặc coordinated buys để mua 50-80%
/// supply qua nhiều ví ẩn trong genesis window (creation slot + N slots).
///
/// Ba tầng phát hiện:
///   1. Supply Concentration: tổng % supply bought > threshold → FAIL
///   2. Single Whale: 1 ví mua > 20% supply → FAIL
///   3. Wallet Clustering: nhiều ví mua cùng funding source → FAIL
///
/// Slot-aware: chỉ count buys trong creation_slot + genesis_slot_window.
/// Memory-safe: auto cleanup entries older than 120 seconds.

use crate::*;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use solana_sdk::pubkey::Pubkey;
use std::time::Instant;

const MODULE_NAME: &str = "GENESIS_DETECTOR";

/// Global tracker for genesis buy activity per mint.
/// Key: mint pubkey → Value: GenesisTrackingData
static GENESIS_TRACKER: Lazy<DashMap<Pubkey, GenesisTrackingData>> = Lazy::new(DashMap::new);

// ══════════════════════════════════════════════════════════════════════
// Public API — called from handle_sniper_event.rs
// ══════════════════════════════════════════════════════════════════════

/// Register a new mint for genesis tracking.
/// Called when a MintEvent is detected in the gRPC stream.
///
/// Arguments:
///   - `mint`: token mint address
///   - `creator`: dev/creator wallet
///   - `total_supply`: total token supply
///   - `creation_slot`: the slot in which this token was created
pub fn genesis_register_mint(
    mint: Pubkey,
    creator: Pubkey,
    total_supply: u64,
    creation_slot: u64,
) {
    if !*GENESIS_FILTER_ENABLED {
        return;
    }

    GENESIS_TRACKER.insert(
        mint,
        GenesisTrackingData::new(creation_slot, total_supply, creator),
    );
}

/// Record a buy event for genesis analysis.
/// Called for every PumpfunBuyEvent on tracked mints.
///
/// Arguments:
///   - `mint`: token mint address
///   - `buyer`: buyer wallet address
///   - `token_amount`: amount of tokens purchased (raw units)
///   - `sol_amount`: SOL spent (lamports)
///   - `slot`: the slot in which this buy occurred
pub fn genesis_record_buy(
    mint: Pubkey,
    buyer: Pubkey,
    token_amount: u64,
    sol_amount: u64,
    slot: u64,
) {
    if !*GENESIS_FILTER_ENABLED {
        return;
    }

    if let Some(mut data) = GENESIS_TRACKER.get_mut(&mint) {
        // Only track buys within the genesis window
        if !data.is_within_genesis_window(slot, *GENESIS_SLOT_WINDOW) {
            return;
        }

        // Enforce tracking limit to prevent memory issues
        if data.buy_records.len() >= *MAX_GENESIS_BUY_TRACKING {
            return;
        }

        data.buy_records.push(GenesisBuyRecord {
            buyer,
            token_amount,
            sol_amount,
            slot,
        });
    }
}

/// Run the genesis filter check for a given mint.
/// Performs three checks:
///   1. Total supply concentration
///   2. Single whale detection
///   3. Wallet clustering (same-slot coordinated buys)
///
/// Returns a FilterResult indicating pass/fail with risk score.
pub fn genesis_check(mint: Pubkey) -> FilterResult {
    if !*GENESIS_FILTER_ENABLED {
        return FilterResult::pass(MODULE_NAME);
    }

    let data = match GENESIS_TRACKER.get(&mint) {
        Some(d) => d.clone(),
        None => return FilterResult::pass(MODULE_NAME),
    };

    // Skip if no buys recorded yet
    if data.buy_records.is_empty() {
        return FilterResult::pass(MODULE_NAME);
    }

    // ── Check 1: Total supply concentration ──
    let genesis_buy_pct = data.genesis_buy_percent();

    if genesis_buy_pct > *MAX_GENESIS_BUY_PERCENT {
        let reason = format!(
            "Genesis supply concentration: {:.1}% > {:.1}% limit | {} unique buyers, {} buys in {} slots",
            genesis_buy_pct,
            *MAX_GENESIS_BUY_PERCENT,
            data.unique_buyer_count(),
            data.buy_records.len(),
            data.buy_records.last().map(|r| r.slot.saturating_sub(data.creation_slot)).unwrap_or(0),
        );
        info!("[{}] REJECT MINT: {} | {}", MODULE_NAME, mint, reason);
        return FilterResult::fail(MODULE_NAME, reason, 50.0);
    }

    // ── Check 2: Single whale detection ──
    let largest_buyer_pct = data.largest_single_buyer_percent();

    if largest_buyer_pct > *MAX_SINGLE_WALLET_PERCENT {
        let reason = format!(
            "Single whale bought {:.1}% > {:.1}% limit | total genesis: {:.1}%",
            largest_buyer_pct, *MAX_SINGLE_WALLET_PERCENT, genesis_buy_pct,
        );
        info!("[{}] REJECT MINT: {} | {}", MODULE_NAME, mint, reason);
        return FilterResult::fail(MODULE_NAME, reason, 45.0);
    }

    // ── Check 3: Wallet clustering ──
    // Multiple different wallets buying in the same slot = likely coordinated (Jito bundle)
    let non_creator_count = data.non_creator_buyer_count() as u32;

    if non_creator_count >= *MAX_CLUSTERED_WALLETS {
        // Count how many buys occurred in the SAME slot as creation
        let same_slot_buys = data
            .buy_records
            .iter()
            .filter(|r| r.slot == data.creation_slot && r.buyer != data.creator)
            .count();

        if same_slot_buys >= *MAX_CLUSTERED_WALLETS as usize {
            let reason = format!(
                "Clustered genesis buys: {} wallets bought in creation slot (slot {}) | {} total non-creator buyers | {:.1}% supply",
                same_slot_buys,
                data.creation_slot,
                non_creator_count,
                genesis_buy_pct,
            );
            info!("[{}] REJECT MINT: {} | {}", MODULE_NAME, mint, reason);
            return FilterResult::fail(MODULE_NAME, reason, 40.0);
        }

        // Even across multiple slots, too many different wallets is suspicious
        let reason = format!(
            "High genesis buyer count: {} non-creator wallets >= {} limit | {:.1}% supply across {} slots",
            non_creator_count,
            *MAX_CLUSTERED_WALLETS,
            genesis_buy_pct,
            data.buy_records.last().map(|r| r.slot.saturating_sub(data.creation_slot) + 1).unwrap_or(1),
        );
        info!("[{}] WARN MINT: {} | {}", MODULE_NAME, mint, reason);
        return FilterResult::warn(MODULE_NAME, reason, 25.0);
    }

    // ── Soft warning: elevated but below thresholds ──
    if genesis_buy_pct > *MAX_GENESIS_BUY_PERCENT * 0.6 {
        let reason = format!(
            "Elevated genesis buy: {:.1}% (threshold: {:.1}%) | {} buyers",
            genesis_buy_pct,
            *MAX_GENESIS_BUY_PERCENT,
            data.unique_buyer_count(),
        );
        return FilterResult::warn(MODULE_NAME, reason, 15.0);
    }

    FilterResult::pass(MODULE_NAME)
}

// ══════════════════════════════════════════════════════════════════════
// Maintenance
// ══════════════════════════════════════════════════════════════════════

/// Clean up old tracking data for mints older than 300 seconds (5 minutes).
/// Called periodically from a background task to prevent memory leaks.
/// 300s chosen to ensure bundled buys arriving 1-3 minutes after mint
/// are still tracked. Memory cost: ~200 bytes × entries ≈ negligible.
pub fn genesis_cleanup() {
    let cutoff = std::time::Duration::from_secs(300);
    let before = GENESIS_TRACKER.len();
    GENESIS_TRACKER.retain(|_, data| data.created_at.elapsed() < cutoff);
    let after = GENESIS_TRACKER.len();
    if before > after {
        info!(
            "[{}] Cleanup: removed {} stale entries ({} → {})",
            MODULE_NAME,
            before - after,
            before,
            after,
        );
    }
}

/// Get the current number of tracked mints (for monitoring).
pub fn genesis_tracker_size() -> usize {
    GENESIS_TRACKER.len()
}
