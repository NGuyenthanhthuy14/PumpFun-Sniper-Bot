/// Phase 2 — Known CEX Hot Wallet Addresses
///
/// Comprehensive list of known centralized exchange (CEX) hot wallets on Solana.
/// Used by the wallet profiler to detect wallets funded directly from exchanges,
/// which is a common pattern for rug-pull developers who want to remain anonymous.
///
/// Sources: Solscan labels, public exchange documentation, community databases.
/// Last updated: 2026-05-12

use solana_sdk::{pubkey, pubkey::Pubkey};

/// Binance hot wallets (main exchange + internal transfers)
pub const BINANCE_WALLETS: &[Pubkey] = &[
    pubkey!("2ojv9BAiHUrvsm9gxDe7fJSzbNZSJcxZvf8dqmWGHG8S"),
    pubkey!("5tzFkiKscXHK5ZXCGbXZxdw7gTjjD1mBwuoFbhUvuAi9"),
    pubkey!("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
    pubkey!("3yFwqXBfZY4jBVUafQ1YEXw189y2dN3V5KQq9uzBDy1E"),
    pubkey!("HN7cABqLq46Es1jh92dQQisAq662SmxELLLsHHe4YWrH"),
    pubkey!("F4TjXLaQPMEnMbZJ8GDR49GvWJz29dsFrMKRWB3JEFLi"),
    pubkey!("CuieVDEDtLo7FypA946cYHVH6QfADSNBp22ND1LPAazq"),
];

/// Bybit hot wallets
pub const BYBIT_WALLETS: &[Pubkey] = &[
    pubkey!("AC5RDfQFmDS1deWZos921JfqscXdByf8BKHs5ACWjtW2"),
    pubkey!("GJtJuWD9qYcCkrwMBmtY1tpapV1sKfB2zUv9Q4aqpnGd"),
];

/// OKX hot wallets
pub const OKX_WALLETS: &[Pubkey] = &[
    pubkey!("5VCwKtCXgCDuQosQe3MbMGrFfJzWidgYyMEpmxCa87xC"),
    pubkey!("JA5cjkRJ1euVi9xLWsCJVzsRzEkT8vcC4rqw9sVAo5d6"),
    pubkey!("ASTyfSima4LLAdDgoFGkgqoKowG1LZFDr9fAQrg7iaJZ"),
];

/// KuCoin hot wallets
pub const KUCOIN_WALLETS: &[Pubkey] = &[
    pubkey!("BmFdpraQhkiDQE6SnfG5PkCNJGKTqbGQBgYPCFSWFWuy"),
    pubkey!("6tj1THeeDAXcBFAHRp6vkAb6CKs5eAtNqVzgbL5YqajV"),
];

/// Gate.io hot wallets
pub const GATE_WALLETS: &[Pubkey] = &[
    pubkey!("u6PJ8DtQuPFnfmwHbGFULQ4u4EgjDiyYKjVEsynXq2w"),
    pubkey!("7hUdUTkJLwdcmt3jSEkwkTR1L2gD4MZRp1r1nVhhmNRm"),
];

/// Coinbase hot wallets
pub const COINBASE_WALLETS: &[Pubkey] = &[
    pubkey!("GJRs4FwHtemZ5ZE9x3FNvJ8TMwitKTh21yxdRPqn7npE"),
    pubkey!("2AQdpHJ2JpcEgPiATUXjQxA8QmafFegfQwSLWSprPicm"),
    pubkey!("H8sMJSCQxfKiFTCfDR3DUMLPwcRbM61LGFJ8N4dK3WjS"),
];

/// Kraken hot wallets
pub const KRAKEN_WALLETS: &[Pubkey] = &[
    pubkey!("FWznbcNXWQuHTawe9RxvQ2LdCENssh12dsznf4RiouN5"),
    pubkey!("krakqkWHEjzJNfDwhmLz8PYSkPbJ1BnDu1X4ynNF3gM"),
];

/// Huobi/HTX hot wallets
pub const HUOBI_WALLETS: &[Pubkey] = &[
    pubkey!("88xTWZMeKFaPH714feFMoTfyV6gDeFHDn11dpLnDkCdK"),
    pubkey!("HKq5bqkgUJoa5XN3vZM6YUBiDfFnNqFMS86bDQPAHGMJ"),
];

/// Crypto.com hot wallets
pub const CRYPTO_COM_WALLETS: &[Pubkey] = &[
    pubkey!("AobVSwdW9BbpMdJvTqeCN4hPAmh4rHm7vwLnQ5ATbo3k"),
    pubkey!("6FEVkH17P9y8Q9aCkDdPcMDjvj7SVxrTETaYEm8f51S3"),
];

// ══════════════════════════════════════════════════════════════════════
// Lookup API
// ══════════════════════════════════════════════════════════════════════

/// Check if a given pubkey is a known CEX hot wallet.
/// Returns Some(exchange_name) if matched, None otherwise.
pub fn identify_cex_wallet(pubkey: &Pubkey) -> Option<&'static str> {
    if BINANCE_WALLETS.contains(pubkey) {
        return Some("Binance");
    }
    if BYBIT_WALLETS.contains(pubkey) {
        return Some("Bybit");
    }
    if OKX_WALLETS.contains(pubkey) {
        return Some("OKX");
    }
    if KUCOIN_WALLETS.contains(pubkey) {
        return Some("KuCoin");
    }
    if GATE_WALLETS.contains(pubkey) {
        return Some("Gate.io");
    }
    if COINBASE_WALLETS.contains(pubkey) {
        return Some("Coinbase");
    }
    if KRAKEN_WALLETS.contains(pubkey) {
        return Some("Kraken");
    }
    if HUOBI_WALLETS.contains(pubkey) {
        return Some("Huobi/HTX");
    }
    if CRYPTO_COM_WALLETS.contains(pubkey) {
        return Some("Crypto.com");
    }
    None
}

/// Total number of known CEX wallets in the database.
pub fn total_known_cex_wallets() -> usize {
    BINANCE_WALLETS.len()
        + BYBIT_WALLETS.len()
        + OKX_WALLETS.len()
        + KUCOIN_WALLETS.len()
        + GATE_WALLETS.len()
        + COINBASE_WALLETS.len()
        + KRAKEN_WALLETS.len()
        + HUOBI_WALLETS.len()
        + CRYPTO_COM_WALLETS.len()
}
