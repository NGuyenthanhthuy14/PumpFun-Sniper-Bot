/// Phase 2 — Dynamic Wallet Manager via Telegram
///
/// Stores multiple wallets in `wallets.json` on disk.
/// Commands:
///   /wallets         — show wallet list + balance of active wallet
///   /generate        — generate a brand-new Solana keypair
///   /import_key      — enter "waiting for private key" mode
///   /select_N        — select wallet N as the active trading wallet
///   /delete_N        — remove wallet N from list
///   /show_key_N      — reveal private key of wallet N
///
/// When a wallet is selected, the bot writes it into `.env` as PRIVATE_KEY
/// and then **restarts itself** so the Lazy<Keypair> statics pick up the new key.

use serde::{Deserialize, Serialize};
use solana_sdk::signer::{keypair::Keypair, Signer};
use std::path::Path;

const WALLETS_FILE: &str = "wallets.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEntry {
    /// Human-readable label (auto-generated or user-provided)
    pub label: String,
    /// Base58-encoded private key (full 64-byte keypair)
    pub private_key: String,
    /// Base58-encoded public key (derived, stored for quick display)
    pub pubkey: String,
    /// Whether this wallet is currently selected for trading
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WalletStore {
    pub wallets: Vec<WalletEntry>,
}

impl WalletStore {
    /// Load from disk, or create empty store
    pub fn load() -> Self {
        let path = Path::new(WALLETS_FILE);
        if path.exists() {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(store) = serde_json::from_str::<WalletStore>(&data) {
                    return store;
                }
            }
        }
        // If no file exists, seed with the current PRIVATE_KEY from .env
        let mut store = WalletStore::default();
        if let Ok(pk) = std::env::var("PRIVATE_KEY") {
            if !pk.is_empty() {
                let kp = Keypair::from_base58_string(&pk);
                store.wallets.push(WalletEntry {
                    label: "Default".to_string(),
                    private_key: pk,
                    pubkey: kp.pubkey().to_string(),
                    active: true,
                });
                store.save();
            }
        }
        store
    }

    /// Persist to disk
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(WALLETS_FILE, json);
        }
    }

    /// Generate a new random wallet and append it
    pub fn generate(&mut self) -> (String, String, usize) {
        let kp = Keypair::new();
        let pk = bs58::encode(kp.to_bytes()).into_string();
        let pubkey = kp.pubkey().to_string();
        let idx = self.wallets.len() + 1;
        let label = format!("Wallet {}", idx);
        self.wallets.push(WalletEntry {
            label: label.clone(),
            private_key: pk,
            pubkey: pubkey.clone(),
            active: false,
        });
        self.save();
        (pubkey, label, self.wallets.len())
    }

    /// Import a wallet from a base58 private key string
    pub fn import(&mut self, private_key_b58: &str) -> Result<(String, usize), String> {
        // Validate the key
        let bytes = bs58::decode(private_key_b58)
            .into_vec()
            .map_err(|e| format!("Invalid base58: {}", e))?;
        if bytes.len() != 64 {
            return Err(format!("Expected 64 bytes, got {}", bytes.len()));
        }
        let kp = Keypair::from_bytes(&bytes)
            .map_err(|e| format!("Invalid keypair: {}", e))?;

        // Check for duplicates
        let pubkey = kp.pubkey().to_string();
        if self.wallets.iter().any(|w| w.pubkey == pubkey) {
            return Err("Wallet already exists in your list".to_string());
        }

        let idx = self.wallets.len() + 1;
        self.wallets.push(WalletEntry {
            label: format!("Imported {}", idx),
            private_key: private_key_b58.to_string(),
            pubkey: pubkey.clone(),
            active: false,
        });
        self.save();
        Ok((pubkey, self.wallets.len()))
    }

    /// Select wallet at 1-based index as active (deactivating others)
    /// Returns (private_key, pubkey) of the selected wallet
    pub fn select(&mut self, index: usize) -> Option<(String, String)> {
        if index == 0 || index > self.wallets.len() {
            return None;
        }
        for w in self.wallets.iter_mut() {
            w.active = false;
        }
        self.wallets[index - 1].active = true;
        self.save();
        let w = &self.wallets[index - 1];
        Some((w.private_key.clone(), w.pubkey.clone()))
    }

    /// Delete wallet at 1-based index
    pub fn delete(&mut self, index: usize) -> Result<WalletEntry, String> {
        if index == 0 || index > self.wallets.len() {
            return Err("Invalid wallet index".to_string());
        }
        let entry = &self.wallets[index - 1];
        if entry.active {
            return Err("Cannot delete the active wallet. Select another wallet first.".to_string());
        }
        let removed = self.wallets.remove(index - 1);
        self.save();
        Ok(removed)
    }

    /// Get wallet at 1-based index
    pub fn get(&self, index: usize) -> Option<&WalletEntry> {
        if index == 0 || index > self.wallets.len() {
            return None;
        }
        Some(&self.wallets[index - 1])
    }

    /// Get the currently active wallet
    pub fn active(&self) -> Option<&WalletEntry> {
        self.wallets.iter().find(|w| w.active)
    }

    /// Build a display string for the wallet list
    pub fn display_list(&self) -> String {
        if self.wallets.is_empty() {
            return "No wallets configured.".to_string();
        }
        let mut s = String::new();
        for (i, w) in self.wallets.iter().enumerate() {
            let marker = if w.active { " ✅" } else { "" };
            s.push_str(&format!(
                "{}. <code>{}</code>{}\n",
                i + 1,
                w.pubkey,
                marker
            ));
        }
        s
    }
}

pub fn switch_active_wallet_and_restart(private_key: &str) -> Result<(), String> {
    let pk_clone = private_key.to_string();
    
    // Schedule a self-restart after a short delay
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(1));
        eprintln!("🔄 [WALLET_MANAGER] Recreating Nonces & Restarting bot with new wallet...");
        
        let script = format!(r#"#!/bin/bash
cd ~/phase2_bot
# Close old nonces using the CURRENT .env
echo -e "3\nyes\n0\n" | ./target/release/nonce-manager

# Update .env with new private key
sed -i 's/^PRIVATE_KEY=.*/PRIVATE_KEY={}/' .env

# Create new nonces with the NEW .env
export $(cat .env | xargs)
echo -e "1\n40\n0\n" | ./target/release/nonce-manager

# Kill current bot and start new one
pkill -f sniper-mode
screen -X -S bot quit 2>/dev/null
screen -dmS bot bash -c "export \$(cat .env | xargs) && ./target/release/sniper-mode 2>&1 | tee -a /tmp/bot_live.log"
"#, pk_clone);

        let _ = std::fs::write("/tmp/switch_wallet.sh", script);
        let _ = std::process::Command::new("bash")
            .arg("/tmp/switch_wallet.sh")
            .spawn();
            
        std::process::exit(0);
    });

    Ok(())
}
