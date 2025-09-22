use std::fs;
use once_cell::sync::Lazy;
use serde::Deserialize;

pub mod toml_config;
pub mod bot_config;

pub use toml_config::*;
pub use bot_config::*;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub wallet_credential: WalletCredentialConfig,
  pub connection_config: ConnectionConfig,
  pub relayer: RelayerConfig,
  pub buy_setting: BuySetting,
  pub slippage: u32,
}

pub static CONFIG: Lazy<Config> = Lazy::new(||{
  let content = fs::read_to_string("Config.toml").expect("Failed to read Config.toml file");
  toml::from_str(&content).expect("Failed to parse config file.")
});