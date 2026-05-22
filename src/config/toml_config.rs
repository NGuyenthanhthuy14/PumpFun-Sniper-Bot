use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ModeConfig {
    pub is_dev_mode: bool,
    pub buy_tx_counter: i32,
}

#[derive(Debug, Deserialize)]
pub struct WalletCredentialConfig {
    pub private_key: String,
}

#[derive(Debug, Deserialize)]
pub struct ConnectionConfig {
    pub grpc_endpoint: String,
    pub grpc_token: String,
    pub rpc_endpoint: String,
    pub bnb_rpc_endpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct BuySetting {
    pub buy_amount_sol: f64,
    pub dynamic_buy_amount_mode: bool,
    pub loss_sequence: u32,
    pub profit_sequence: u32,
    pub loss_multiply: f64,
    pub profit_multiply: f64,
    pub max_buy_amount_multiply: f64,
    pub min_buy_amount_multiply: f64,
}

#[derive(Debug, Deserialize)]
pub struct SellSetting {
    pub stop_loss: f64,
    pub tp_trailing: f64,
    pub trailing_stop: f64,
    pub no_activity_time: u64,
}

#[derive(Debug, Deserialize)]
pub struct SlippageConfig {
    pub slippage_percent: u32,
}

#[derive(Debug, Deserialize)]
pub struct LandingServiceConfig {
    pub landing_service: String,
    pub zero_slot_api_key: String,
    pub helius_api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct FeeConfig {
    pub buy_compute_unit_limit: u64,
    pub buy_micro_lamports: u64,
    pub sell_micro_lamports: f64,
    pub zero_slot_fee: f64,
    pub helius_fee: f64,
}

#[derive(Debug, Deserialize)]
pub struct SimulationSetting {
    pub buy_amount_sol: f64,
    pub stop_loss: f64,
    pub tp_trailing: f64,
    pub trailing_stop: f64,
    pub confirmation_delay_ms: u64,
    pub landing_service: String,
    pub buy_compute_unit_limit: u64,
    pub buy_micro_lamports: u64,
    pub sell_compute_unit_limit: u64,
    pub sell_micro_lamports: u64,
    pub zero_slot_fee: f64,
    pub helius_fee: f64,
}

#[derive(Debug, Deserialize)]
pub struct FilterSetting {
    pub rug_detect: bool,
    pub bundle_tx_limit: i32,
    pub volume_filter: bool,
    pub min_volume_limit_sol: i32,
    pub market_cap_filter: bool,
    pub min_market_cap_limit_sol: i32,
    pub max_token_holder_filter: bool,
    pub max_token_holder_limit: u64,
}

// ══════════════════════════════════════════════════════════════════════
// Phase 2 — Anti-Rug Genesis Filter Configuration
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize, Clone)]
pub struct GenesisFilterConfig {
    /// Master toggle for genesis bundle detection
    pub enabled: bool,
    /// Max % of supply bought in creation block before aborting (default: 50.0)
    pub max_genesis_buy_percent: f64,
    /// Max number of clustered wallets in genesis block (default: 3)
    pub max_clustered_wallets: u32,
    /// Number of buy events to track per mint for genesis analysis
    pub max_genesis_buy_tracking: usize,
    /// Number of slots after creation to include in genesis window (default: 3)
    pub genesis_slot_window: u64,
    /// Max % a single wallet can buy before flagging (default: 20.0)
    pub max_single_wallet_percent: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WalletProfilerConfig {
    /// Master toggle for dev wallet profiling
    pub enabled: bool,
    /// Minimum wallet age in hours (default: 24)
    pub min_wallet_age_hours: u64,
    /// Minimum historical TX count (default: 10)
    pub min_historical_tx_count: u64,
    /// Block tokens from CEX-funded fresh wallets
    pub block_cex_funded: bool,
    /// RPC call timeout in milliseconds (default: 500)
    pub rpc_timeout_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetadataCheckerConfig {
    /// Master toggle for metadata verification
    pub enabled: bool,
    /// Require non-empty metadata URI
    pub require_metadata_uri: bool,
    /// Minimum token name length (default: 2)
    pub min_name_length: usize,
    /// Minimum token symbol length (default: 2)
    pub min_symbol_length: usize,
    /// Action when metadata is empty: "skip" | "warn" | "allow"
    pub metadata_empty_action: String,
    /// Whether to fetch URI content and parse Metaplex JSON (default: true)
    pub fetch_uri_content: bool,
    /// Timeout for URI HTTP fetch in milliseconds (default: 2000)
    pub uri_timeout_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RiskScoringConfig {
    /// Maximum total risk score before rejecting token (default: 70.0)
    pub max_total_risk_score: f64,
    /// Enable dynamic position sizing based on risk score (default: true)
    pub enable_dynamic_sizing: bool,
    /// Minimum buy multiplier when risk is high (default: 0.3)
    pub min_buy_multiplier: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FilterLogConfig {
    /// Enable CSV audit logging for filter decisions
    pub enabled: bool,
    /// Directory for filter audit logs (default: "filter_logs")
    pub log_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BuyGuardConfig {
    /// Maximum number of open positions at any time (default: 3)
    pub max_open_positions: usize,
    /// Minimum milliseconds between consecutive BUY transactions (default: 500)
    pub buy_cooldown_ms: u64,
    /// Stop buying if wallet SOL balance drops below this amount (default: 0.05)
    pub min_sol_balance: f64,
}