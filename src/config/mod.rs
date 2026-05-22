use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;

pub mod bot_config;
pub mod toml_config;
pub mod trade_setting;

pub use bot_config::*;
pub use toml_config::*;
pub use trade_setting::*;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mode: ModeConfig,
    pub wallet_config: WalletCredentialConfig,
    pub connection_config: ConnectionConfig,
    pub buy_setting: BuySetting,
    pub sell_setting: SellSetting,
    pub landing_service_config: LandingServiceConfig,
    pub slippage_config: SlippageConfig,
    pub fee_config: FeeConfig,
    pub simulation_setting: SimulationSetting,

    // Phase 2 — Anti-Rug Genesis Filter configs (optional, with defaults)
    #[serde(default = "default_genesis_filter")]
    pub genesis_filter: GenesisFilterConfig,
    #[serde(default = "default_wallet_profiler")]
    pub wallet_profiler: WalletProfilerConfig,
    #[serde(default = "default_metadata_checker")]
    pub metadata_checker: MetadataCheckerConfig,
    #[serde(default = "default_risk_scoring")]
    pub risk_scoring: RiskScoringConfig,
    #[serde(default = "default_filter_log")]
    pub filter_log: FilterLogConfig,
    #[serde(default = "default_buy_guard")]
    pub buy_guard: BuyGuardConfig,
}

// ── Phase 2 config defaults (used when TOML sections are absent) ──

fn default_genesis_filter() -> GenesisFilterConfig {
    GenesisFilterConfig {
        enabled: false, // Disabled by default — enable after calibration
        max_genesis_buy_percent: 50.0,
        max_clustered_wallets: 3,
        max_genesis_buy_tracking: 10,
        genesis_slot_window: 3,
        max_single_wallet_percent: 20.0,
    }
}

fn default_wallet_profiler() -> WalletProfilerConfig {
    WalletProfilerConfig {
        enabled: false, // Disabled by default — enable after calibration
        min_wallet_age_hours: 24,
        min_historical_tx_count: 10,
        block_cex_funded: true,
        rpc_timeout_ms: 500,
    }
}

fn default_metadata_checker() -> MetadataCheckerConfig {
    MetadataCheckerConfig {
        enabled: false, // Disabled by default — enable after calibration
        require_metadata_uri: true,
        min_name_length: 2,
        min_symbol_length: 2,
        metadata_empty_action: "skip".to_string(),
        fetch_uri_content: true,
        uri_timeout_ms: 2000,
    }
}

fn default_risk_scoring() -> RiskScoringConfig {
    RiskScoringConfig {
        max_total_risk_score: 70.0,
        enable_dynamic_sizing: true,
        min_buy_multiplier: 0.3,
    }
}

fn default_filter_log() -> FilterLogConfig {
    FilterLogConfig {
        enabled: true,
        log_dir: "filter_logs".to_string(),
    }
}

fn default_buy_guard() -> BuyGuardConfig {
    BuyGuardConfig {
        max_open_positions: 3,
        buy_cooldown_ms: 500,
        min_sol_balance: 0.05,
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    // let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "/app/Config.toml".to_string());
    // let content = fs::read_to_string(&config_path).expect("Failed to read config file");
    
    /////////None deploy, just test on local
    let content = fs::read_to_string("Config.toml").expect("Failed to read Config.toml file");

    toml::from_str(&content).expect("Failed to parse config file")
});
