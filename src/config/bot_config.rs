use crate::*;
use once_cell::sync::Lazy;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    commitment_config::CommitmentLevel,
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

//Bot mode
pub static DEV_MODE: Lazy<bool> = Lazy::new(|| CONFIG.mode.is_dev_mode);
pub static BUY_TX_COUNTER: Lazy<AtomicI32> =
    Lazy::new(|| AtomicI32::new(CONFIG.mode.buy_tx_counter));

pub fn decrese_buy_tx_remain_counter() {
    BUY_TX_COUNTER.fetch_sub(1, Ordering::SeqCst);
}

pub fn get_buy_tx_remain_counter() -> i32 {
    BUY_TX_COUNTER.load(Ordering::SeqCst)
}

//Wallet key
pub static SIGNER_KEYPAIR: Lazy<Keypair> = Lazy::new(|| {
    let pk = std::env::var("PRIVATE_KEY").unwrap_or_else(|_| CONFIG.wallet_config.private_key.clone());
    Keypair::from_base58_string(&pk)
});

pub static SIGNER_PUBKEY: Lazy<Pubkey> = Lazy::new(|| {
    let pk = std::env::var("PRIVATE_KEY").unwrap_or_else(|_| CONFIG.wallet_config.private_key.clone());
    let wallet = Keypair::from_base58_string(&pk);
    wallet.pubkey()
});

//RPC endpoint
pub static RPC_ENDPOINT: Lazy<String> = Lazy::new(|| CONFIG.connection_config.rpc_endpoint.clone());
pub static RPC_CLIENT: Lazy<Arc<RpcClient>> = Lazy::new(|| {
    Arc::new(RpcClient::new_with_commitment(
        CONFIG.connection_config.rpc_endpoint.clone(),
        CommitmentConfig {
            commitment: CommitmentLevel::Processed,
        },
    ))
});
pub static GRPC_ENDPOINT: Lazy<String> =
    Lazy::new(|| CONFIG.connection_config.grpc_endpoint.clone());
pub static GRPC_TOKEN: Lazy<String> = Lazy::new(|| CONFIG.connection_config.grpc_token.clone());
pub static BNB_RPC_ENDPOINT: Lazy<String> =
    Lazy::new(|| CONFIG.connection_config.bnb_rpc_endpoint.clone());

//Buy setting
pub static BUY_AMOUNT_SOL: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.buy_amount_sol);
pub static DYNAMIC_BUY_AMOUNT_MODE: Lazy<bool> = Lazy::new(|| CONFIG.buy_setting.dynamic_buy_amount_mode);
pub static LOSS_SEQUENCE: Lazy<u32> = Lazy::new(|| CONFIG.buy_setting.loss_sequence.max(1));
pub static PROFIT_SEQUENCE: Lazy<u32> = Lazy::new(|| CONFIG.buy_setting.profit_sequence.max(1));
pub static LOSS_MULTIPLY: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.loss_multiply.clamp(0.01, 1.0));
pub static PROFIT_MULTIPLY: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.profit_multiply.max(1.0));
pub static MAX_BUY_AMOUNT_MULTIPLY: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.max_buy_amount_multiply.max(0.01));
pub static MIN_BUY_AMOUNT_MULTIPLY: Lazy<f64> = Lazy::new(|| CONFIG.buy_setting.min_buy_amount_multiply.clamp(0.001, 1.0));

//Slippage
pub static SLIPPAGE: Lazy<f64> =
    Lazy::new(|| 1.0 + CONFIG.slippage_config.slippage_percent as f64 / 100.0);

//Landing service config
pub static LANDING_SERVICE: Lazy<String> =
    Lazy::new(|| CONFIG.landing_service_config.landing_service.clone());
pub static ZERO_SLOT_API_KEY: Lazy<String> = Lazy::new(|| CONFIG.landing_service_config.zero_slot_api_key.clone());
pub static HELIUS_API_KEY: Lazy<String> = Lazy::new(|| CONFIG.landing_service_config.helius_api_key.clone());

pub static ZERO_SLOT_ENDPOINT: Lazy<String> = Lazy::new(|| format!("http://de1.0slot.trade?api-key={}", *ZERO_SLOT_API_KEY));
pub static HELIUS_ENDPOINT: Lazy<String> = Lazy::new(|| format!("http://fra-sender.helius-rpc.com/fast?api-key={}", *HELIUS_API_KEY));

//Fee
pub static BUY_COMPUTE_UNIT_LIMIT: Lazy<u64> =
    Lazy::new(|| CONFIG.fee_config.buy_compute_unit_limit);
pub static BUY_MICRO_LAMPORTS: Lazy<u64> = Lazy::new(|| CONFIG.fee_config.buy_micro_lamports);
pub static SELL_MICRO_LAMPORTS: Lazy<f64> = Lazy::new(|| CONFIG.fee_config.sell_micro_lamports);
pub static ZERO_SLOT_FEE: Lazy<f64> = Lazy::new(|| CONFIG.fee_config.zero_slot_fee);
pub static HELIUS_FEE: Lazy<f64> = Lazy::new(|| CONFIG.fee_config.helius_fee);

// ══════════════════════════════════════════════════════════════════════
// Phase 2 — Anti-Rug Genesis Filter static config
// ══════════════════════════════════════════════════════════════════════

// Genesis Bundle Detection
pub static GENESIS_FILTER_ENABLED: Lazy<bool> = Lazy::new(|| CONFIG.genesis_filter.enabled);
pub static MAX_GENESIS_BUY_PERCENT: Lazy<f64> = Lazy::new(|| CONFIG.genesis_filter.max_genesis_buy_percent);
pub static MAX_CLUSTERED_WALLETS: Lazy<u32> = Lazy::new(|| CONFIG.genesis_filter.max_clustered_wallets);
pub static MAX_GENESIS_BUY_TRACKING: Lazy<usize> = Lazy::new(|| CONFIG.genesis_filter.max_genesis_buy_tracking);
pub static GENESIS_SLOT_WINDOW: Lazy<u64> = Lazy::new(|| CONFIG.genesis_filter.genesis_slot_window);
pub static MAX_SINGLE_WALLET_PERCENT: Lazy<f64> = Lazy::new(|| CONFIG.genesis_filter.max_single_wallet_percent);

// Wallet Profiler
pub static WALLET_PROFILER_ENABLED: Lazy<bool> = Lazy::new(|| CONFIG.wallet_profiler.enabled);
pub static MIN_WALLET_AGE_HOURS: Lazy<u64> = Lazy::new(|| CONFIG.wallet_profiler.min_wallet_age_hours);
pub static MIN_HISTORICAL_TX_COUNT: Lazy<u64> = Lazy::new(|| CONFIG.wallet_profiler.min_historical_tx_count);
pub static BLOCK_CEX_FUNDED: Lazy<bool> = Lazy::new(|| CONFIG.wallet_profiler.block_cex_funded);
pub static WALLET_RPC_TIMEOUT_MS: Lazy<u64> = Lazy::new(|| CONFIG.wallet_profiler.rpc_timeout_ms);

// Metadata Checker
pub static METADATA_CHECKER_ENABLED: Lazy<bool> = Lazy::new(|| CONFIG.metadata_checker.enabled);
pub static REQUIRE_METADATA_URI: Lazy<bool> = Lazy::new(|| CONFIG.metadata_checker.require_metadata_uri);
pub static MIN_NAME_LENGTH: Lazy<usize> = Lazy::new(|| CONFIG.metadata_checker.min_name_length);
pub static MIN_SYMBOL_LENGTH: Lazy<usize> = Lazy::new(|| CONFIG.metadata_checker.min_symbol_length);
pub static METADATA_EMPTY_ACTION: Lazy<String> = Lazy::new(|| CONFIG.metadata_checker.metadata_empty_action.clone());
pub static FETCH_URI_CONTENT: Lazy<bool> = Lazy::new(|| CONFIG.metadata_checker.fetch_uri_content);
pub static URI_TIMEOUT_MS: Lazy<u64> = Lazy::new(|| CONFIG.metadata_checker.uri_timeout_ms);

// Risk Scoring
pub static MAX_TOTAL_RISK_SCORE: Lazy<f64> = Lazy::new(|| CONFIG.risk_scoring.max_total_risk_score);
pub static ENABLE_DYNAMIC_SIZING: Lazy<bool> = Lazy::new(|| CONFIG.risk_scoring.enable_dynamic_sizing);
pub static MIN_BUY_MULTIPLIER: Lazy<f64> = Lazy::new(|| CONFIG.risk_scoring.min_buy_multiplier);

// Filter Logging
pub static FILTER_LOG_ENABLED: Lazy<bool> = Lazy::new(|| CONFIG.filter_log.enabled);
pub static FILTER_LOG_DIR: Lazy<String> = Lazy::new(|| CONFIG.filter_log.log_dir.clone());

// ══════════════════════════════════════════════════════════════════════
// Phase 2 — Buy Guard (Rate Limiter + Position Control)
// ══════════════════════════════════════════════════════════════════════
//
// Prevents the bot from draining SOL by rapid-fire buying.
// Three safety mechanisms:
//   1. Max concurrent open positions (default: 3)
//   2. Cooldown between consecutive buys (default: 500ms)
//   3. Minimum SOL balance floor (default: 0.05 SOL)

pub static MAX_OPEN_POSITIONS: Lazy<usize> = Lazy::new(|| CONFIG.buy_guard.max_open_positions);
pub static BUY_COOLDOWN_MS: Lazy<u64> = Lazy::new(|| CONFIG.buy_guard.buy_cooldown_ms);
pub static MIN_SOL_BALANCE: Lazy<f64> = Lazy::new(|| CONFIG.buy_guard.min_sol_balance);

use std::sync::atomic::AtomicU64;

/// Tracks the number of currently open positions (bought but not yet sold).
pub static OPEN_POSITION_COUNT: AtomicI32 = AtomicI32::new(0);

/// Tracks the timestamp (ms) of the last BUY submission for cooldown.
pub static LAST_BUY_TIMESTAMP_MS: AtomicU64 = AtomicU64::new(0);

/// Increment open position count (called after BUY is submitted)
pub fn increment_open_positions() {
    OPEN_POSITION_COUNT.fetch_add(1, Ordering::SeqCst);
}

/// Decrement open position count (called after SELL is confirmed)
pub fn decrement_open_positions() {
    let prev = OPEN_POSITION_COUNT.fetch_sub(1, Ordering::SeqCst);
    if prev <= 0 {
        // Prevent underflow — clamp to 0
        OPEN_POSITION_COUNT.store(0, Ordering::SeqCst);
    }
}

/// Get current open position count
pub fn get_open_positions() -> i32 {
    OPEN_POSITION_COUNT.load(Ordering::SeqCst)
}

/// Check if a new BUY is allowed by all three guard conditions.
/// Returns (allowed, reason) for logging.
pub fn buy_guard_check() -> (bool, String) {
    // 1. Check max open positions
    let current_positions = get_open_positions();
    let max_pos = *MAX_OPEN_POSITIONS as i32;
    if current_positions >= max_pos {
        return (false, format!(
            "MAX_POSITIONS: {}/{} open positions", current_positions, max_pos
        ));
    }

    // 2. Check cooldown
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let last_buy = LAST_BUY_TIMESTAMP_MS.load(Ordering::SeqCst);
    let cooldown = *BUY_COOLDOWN_MS;
    if last_buy > 0 && now_ms.saturating_sub(last_buy) < cooldown {
        return (false, format!(
            "COOLDOWN: {}ms since last buy (min: {}ms)",
            now_ms.saturating_sub(last_buy), cooldown
        ));
    }

    (true, "OK".to_string())
}

/// Record that a BUY was just submitted (updates timestamp + position count)
pub fn record_buy_submitted() {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    LAST_BUY_TIMESTAMP_MS.store(now_ms, Ordering::SeqCst);
    increment_open_positions();
}