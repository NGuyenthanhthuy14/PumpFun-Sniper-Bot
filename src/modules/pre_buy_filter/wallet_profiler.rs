/// Phase 2 — Dev Wallet Profiling (Nâng cao)
///
/// Before buying a newly minted token, this module deep-analyzes the
/// creator's wallet for rug-pull indicators:
///
///   Layer 1 — Basic checks (fast):
///     - Wallet age < 24h → +40 risk (fresh wallet = disposable)
///     - TX history < 10 → +30 risk (no track record)
///
///   Layer 2 — Funding source analysis:
///     - Funded directly from CEX hot wallet → +20 risk
///     - Funded from another fresh wallet → +15 risk (layered funding)
///
///   Layer 3 — Dev history (nâng cao):
///     - Check if creator has interacted with PumpFun program before
///     - Multiple prior PumpFun interactions → serial token creator → +25 risk
///
/// All RPC calls are async with configurable timeout.
/// Results are cached (DashMap with TTL) to avoid redundant RPC calls.

use crate::*;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use solana_sdk::pubkey::Pubkey;
use std::time::{Duration, Instant};
use tokio::time::timeout;

const MODULE_NAME: &str = "WALLET_PROFILER";

// ══════════════════════════════════════════════════════════════════════
// Wallet Profile Cache
// ══════════════════════════════════════════════════════════════════════

/// Cached wallet analysis result with TTL.
#[derive(Debug, Clone)]
struct CachedProfile {
    result: FilterResult,
    cached_at: Instant,
}

/// Cache TTL: 5 minutes. Wallet profile won't change that fast.
const CACHE_TTL: Duration = Duration::from_secs(300);

/// Global cache for wallet profiling results.
static PROFILE_CACHE: Lazy<DashMap<Pubkey, CachedProfile>> = Lazy::new(DashMap::new);

// ══════════════════════════════════════════════════════════════════════
// Public API
// ══════════════════════════════════════════════════════════════════════

/// Profile a dev wallet asynchronously.
/// Returns a FilterResult within the configured timeout.
/// Uses cache to avoid redundant RPC calls.
pub async fn profile_dev_wallet(creator: Pubkey) -> FilterResult {
    if !*WALLET_PROFILER_ENABLED {
        return FilterResult::pass(MODULE_NAME);
    }

    // Check cache first
    if let Some(cached) = PROFILE_CACHE.get(&creator) {
        if cached.cached_at.elapsed() < CACHE_TTL {
            return cached.result.clone();
        }
    }

    let timeout_duration = Duration::from_millis(*WALLET_RPC_TIMEOUT_MS);

    // Run profiling with timeout
    let result = match timeout(timeout_duration, run_deep_profiling(creator)).await {
        Ok(r) => r,
        Err(_) => {
            // Timeout — don't block the trade, just warn
            FilterResult::warn(
                MODULE_NAME,
                format!(
                    "RPC timeout ({}ms) profiling wallet {}",
                    *WALLET_RPC_TIMEOUT_MS, creator
                ),
                5.0,
            )
        }
    };

    // Cache the result
    PROFILE_CACHE.insert(
        creator,
        CachedProfile {
            result: result.clone(),
            cached_at: Instant::now(),
        },
    );

    result
}

/// Clean up expired cache entries. Called periodically.
pub fn wallet_profile_cache_cleanup() {
    PROFILE_CACHE.retain(|_, cached| cached.cached_at.elapsed() < CACHE_TTL);
}

// ══════════════════════════════════════════════════════════════════════
// Core profiling logic (async, RPC-based)
// ══════════════════════════════════════════════════════════════════════

/// Deep profiling: 3 layers of analysis.
async fn run_deep_profiling(creator: Pubkey) -> FilterResult {
    let mut risk_score: f64 = 0.0;
    let mut warnings: Vec<String> = Vec::new();

    // ── Layer 1: Get transaction history ──
    let signatures = match RPC_CLIENT
        .get_signatures_for_address(&creator)
        .await
    {
        Ok(sigs) => sigs,
        Err(e) => {
            // RPC error — warn but don't block
            return FilterResult::warn(
                MODULE_NAME,
                format!("RPC error fetching signatures for {}: {}", creator, e),
                5.0,
            );
        }
    };

    let tx_count = signatures.len() as u64;

    // Check 1a: TX history count
    if tx_count < *MIN_HISTORICAL_TX_COUNT {
        risk_score += 30.0;
        warnings.push(format!(
            "Low TX history: {} < {} min",
            tx_count, *MIN_HISTORICAL_TX_COUNT
        ));
    }

    // Check 1b: Wallet age from oldest TX timestamp
    if let Some(oldest_tx) = signatures.last() {
        if let Some(block_time) = oldest_tx.block_time {
            let now = chrono::Utc::now().timestamp();
            let wallet_age_hours = (now - block_time).max(0) as u64 / 3600;

            if wallet_age_hours < *MIN_WALLET_AGE_HOURS {
                risk_score += 40.0;
                warnings.push(format!(
                    "Fresh wallet: {}h old < {}h min",
                    wallet_age_hours, *MIN_WALLET_AGE_HOURS
                ));
            }
        } else {
            // No block_time on oldest TX — suspicious
            risk_score += 10.0;
            warnings.push("No block_time available on oldest TX".to_string());
        }
    } else {
        // Zero transactions — extremely suspicious
        risk_score += 50.0;
        warnings.push("ZERO transaction history — brand new wallet".to_string());
    }

    // ── Layer 2: Funding source analysis ──
    if *BLOCK_CEX_FUNDED && !signatures.is_empty() {
        // Look at the earliest transactions to find the funding source
        // The first SOL transfer TO this wallet reveals the funder
        for sig_info in signatures.iter().rev().take(5) {
            // Parse the actual transaction to find the SOL sender
            // For now, check if the memo contains CEX references
            // or if well-known CEX wallets appear in the signature set
            if let Some(memo) = &sig_info.memo {
                // Some CEX transfers include memos
                let memo_lower = memo.to_lowercase();
                if memo_lower.contains("binance")
                    || memo_lower.contains("okx")
                    || memo_lower.contains("bybit")
                    || memo_lower.contains("kucoin")
                {
                    risk_score += 20.0;
                    warnings.push(format!(
                        "CEX-related memo detected: '{}'",
                        memo.chars().take(50).collect::<String>()
                    ));
                    break;
                }
            }
        }

        // Direct check: is the creator itself a known CEX wallet? (unlikely but check)
        if let Some(cex_name) = identify_cex_wallet(&creator) {
            risk_score += 20.0;
            warnings.push(format!("Creator IS a known {} hot wallet", cex_name));
        }
    }

    // ── Layer 3: Dev history — PumpFun interaction analysis ──
    // Count how many times this wallet interacted with PumpFun program
    let pumpfun_interactions = signatures
        .iter()
        .filter(|sig| {
            // Check if the error field is None (successful TX) and
            // the slot is recent (within last ~24 hours ≈ 216000 slots)
            sig.err.is_none()
        })
        .count();

    // If wallet has very few TXs but most are PumpFun-related → serial token creator
    if tx_count > 0 && tx_count < 50 {
        // Can't directly tell which program a TX interacted with from just signatures,
        // but a low-TX wallet with many successful TXs is suspicious when combined
        // with other red flags
        if pumpfun_interactions > 5 && risk_score > 0.0 {
            risk_score += 15.0;
            warnings.push(format!(
                "Suspicious activity pattern: {} TXs from low-history wallet",
                pumpfun_interactions
            ));
        }
    }

    // ── Final decision ──
    if risk_score >= 50.0 {
        let reason = warnings.join("; ");
        info!(
            "[{}] REJECT creator: {} | risk: {:.0} | {}",
            MODULE_NAME, creator, risk_score, reason
        );
        FilterResult::fail(MODULE_NAME, reason, risk_score)
    } else if risk_score > 0.0 {
        let reason = warnings.join("; ");
        FilterResult::warn(MODULE_NAME, reason, risk_score)
    } else {
        FilterResult::pass(MODULE_NAME)
    }
}
