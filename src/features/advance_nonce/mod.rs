use crate::*;
use colored::*;
use serde::{Deserialize, Serialize};
#[allow(deprecated)]
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;
use tokio::sync::OnceCell;
use tokio::time::{Duration, sleep};

const NONCE_ACCOUNTS_PATH: &str = "nonce_accounts.json";
const NONCE_RENT_LAMPORTS: u64 = 1_447_680; // Rent-exempt minimum for nonce account (80 bytes)

// ─── Nonce accounts file ─────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
struct NonceAccountsFile {
    accounts: Vec<String>,
}

fn load_nonce_pubkeys() -> Vec<Pubkey> {
    let path = Path::new(NONCE_ACCOUNTS_PATH);
    if !path.exists() {
        return Vec::new();
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let file: NonceAccountsFile = match serde_json::from_str(&content) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    file.accounts
        .iter()
        .filter_map(|s| Pubkey::from_str(s).ok())
        .collect()
}

fn save_nonce_pubkeys(pubkeys: &[Pubkey]) {
    let file = NonceAccountsFile {
        accounts: pubkeys.iter().map(|p| p.to_string()).collect(),
    };
    let json = serde_json::to_string_pretty(&file).expect("Failed to serialize nonce accounts");
    fs::write(NONCE_ACCOUNTS_PATH, json).expect("Failed to write nonce_accounts.json");
}

// ─── Nonce hash parsing ──────────────────────────────────────────────

/// Parse nonce hash from raw nonce account data.
///
/// Layout (bincode):
///   [0..4]   u32  Versions enum variant (1 = Current)
///   [4..8]   u32  State enum variant (1 = Initialized)
///   [8..40]  [u8;32] Authority pubkey
///   [40..72] [u8;32] Durable nonce hash
///   [72..80] u64  lamports_per_signature
fn parse_nonce_hash_from_data(data: &[u8]) -> Option<Hash> {
    if data.len() < 80 {
        return None;
    }
    let state = u32::from_le_bytes(data[4..8].try_into().ok()?);
    if state != 1 {
        return None;
    }
    let hash_bytes: [u8; 32] = data[40..72].try_into().ok()?;
    Some(Hash::new_from_array(hash_bytes))
}

async fn fetch_nonce_hash(pubkey: &Pubkey) -> Option<Hash> {
    let account = RPC_CLIENT.get_account(pubkey).await.ok()?;
    parse_nonce_hash_from_data(&account.data)
}

// ─── Nonce pool ──────────────────────────────────────────────────────

/// Nonce entry states:
/// 0 = Ready     (has valid hash, available for acquisition)
/// 1 = InUse     (acquired by a transaction, not available)
/// 2 = Refreshing (background task is fetching new hash from RPC)
const STATE_READY: u8 = 0;
const STATE_IN_USE: u8 = 1;
const STATE_REFRESHING: u8 = 2;

struct NonceEntry {
    pubkey: Pubkey,
    nonce_hash: Mutex<Hash>,
    state: AtomicU8,
}

pub struct NoncePool {
    entries: Vec<NonceEntry>,
}

static NONCE_POOL: OnceCell<NoncePool> = OnceCell::const_new();

pub struct AcquiredNonce {
    pub index: usize,
    pub nonce_pubkey: Pubkey,
    pub nonce_hash: Hash,
    pub advance_ix: Instruction,
}

impl NoncePool {
    /// Acquire a ready nonce from the pool. Instant — no RPC calls.
    fn acquire(&self) -> Option<AcquiredNonce> {
        for (i, entry) in self.entries.iter().enumerate() {
            // Only acquire entries in READY state
            if entry
                .state
                .compare_exchange(STATE_READY, STATE_IN_USE, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                let hash = *entry.nonce_hash.lock().unwrap();
                if hash == Hash::default() {
                    // Hash not loaded — put back and skip
                    entry.state.store(STATE_READY, Ordering::SeqCst);
                    continue;
                }
                let advance_ix =
                    system_instruction::advance_nonce_account(&entry.pubkey, &*SIGNER_PUBKEY);
                return Some(AcquiredNonce {
                    index: i,
                    nonce_pubkey: entry.pubkey,
                    nonce_hash: hash,
                    advance_ix,
                });
            }
        }
        None
    }

    /// Spawn a background task to refresh the nonce hash, then mark as READY.
    /// Returns immediately — does NOT block the caller.
    ///
    /// Immediately invalidates the cached hash to prevent reuse of the consumed
    /// nonce, waits for the tx to land on-chain, then retries RPC fetch until a
    /// *new* hash is observed.
    fn spawn_refresh(&self, index: usize) {
        let consumed_hash = if let Some(entry) = self.entries.get(index) {
            let consumed = *entry.nonce_hash.lock().unwrap();
            // Invalidate immediately so acquire() cannot hand out the consumed hash
            *entry.nonce_hash.lock().unwrap() = Hash::default();
            entry.state.store(STATE_REFRESHING, Ordering::SeqCst);
            consumed
        } else {
            return;
        };

        tokio::spawn(async move {
            // Give the transaction time to land on-chain and advance the nonce
            sleep(Duration::from_secs(3)).await;

            if let Some(pool) = NONCE_POOL.get() {
                if let Some(entry) = pool.entries.get(index) {
                    let mut refreshed = false;
                    for attempt in 0..5u64 {
                        if attempt > 0 {
                            sleep(Duration::from_millis(1500)).await;
                        }
                        if let Some(hash) = fetch_nonce_hash(&entry.pubkey).await {
                            if hash != consumed_hash {
                                // Nonce was advanced on-chain — store the new hash
                                *entry.nonce_hash.lock().unwrap() = hash;
                                refreshed = true;
                                break;
                            }
                            // Same hash means the tx hasn't landed yet — retry
                        }
                    }
                    if !refreshed {
                        // All retries exhausted. Hash stays as Hash::default() so
                        // acquire() will skip this slot. The 30-second background
                        // refresh loop will eventually recover it.
                    }
                    entry.state.store(STATE_READY, Ordering::SeqCst);
                }
            }
        });
    }

    /// Release without refreshing (tx was never sent).
    fn release(&self, index: usize) {
        if let Some(entry) = self.entries.get(index) {
            entry.state.store(STATE_READY, Ordering::SeqCst);
        }
    }
}

// ─── Public API ──────────────────────────────────────────────────────

/// Initialize the nonce pool from saved nonce accounts.
/// Fetches current nonce values from RPC. Call once at bot startup.
pub async fn init_nonce_pool() {
    let pubkeys = load_nonce_pubkeys();
    if pubkeys.is_empty() {
        println!(
            "{}",
            "  ⚠ No nonce accounts found. Use CLI option 2 to create them first.".yellow()
        );
        let pool = NoncePool {
            entries: Vec::new(),
        };
        let _ = NONCE_POOL.set(pool);
        return;
    }

    println!(
        "  {} Loading {} nonce accounts...",
        "⏳".yellow(),
        pubkeys.len()
    );

    let mut entries = Vec::with_capacity(pubkeys.len());
    for pk in &pubkeys {
        let hash = fetch_nonce_hash(pk).await.unwrap_or_default();
        if hash == Hash::default() {
            println!("  {} Nonce account {} - failed to fetch", "⚠".red(), pk);
        }
        entries.push(NonceEntry {
            pubkey: *pk,
            nonce_hash: Mutex::new(hash),
            state: AtomicU8::new(STATE_READY),
        });
    }

    let valid = entries
        .iter()
        .filter(|e| *e.nonce_hash.lock().unwrap() != Hash::default())
        .count();
    println!(
        "  {} Nonce pool ready: {}/{} accounts loaded",
        "✅".green(),
        valid,
        entries.len()
    );

    let pool = NoncePool { entries };
    let _ = NONCE_POOL.set(pool);

    // Start background refresh loop to keep nonce hashes warm
    tokio::spawn(async {
        nonce_refresh_loop().await;
    });
}

/// Background loop that periodically re-fetches nonce hashes for READY entries.
/// This keeps the cached hashes fresh in case they expire on-chain.
async fn nonce_refresh_loop() {
    loop {
        sleep(Duration::from_secs(30)).await;
        let pool = match NONCE_POOL.get() {
            Some(p) => p,
            None => continue,
        };
        for entry in &pool.entries {
            // Only refresh entries that are READY (not in-use or already refreshing)
            if entry.state.load(Ordering::SeqCst) == STATE_READY {
                if let Some(hash) = fetch_nonce_hash(&entry.pubkey).await {
                    // Re-check state: entry may have been acquired while the
                    // RPC call was in flight — don't overwrite a newer hash.
                    if entry.state.load(Ordering::SeqCst) == STATE_READY {
                        *entry.nonce_hash.lock().unwrap() = hash;
                    }
                }
            }
        }
    }
}

/// Acquire a nonce from the pool for transaction use.
/// Instant — reads from pre-cached hash, no RPC call.
/// Returns None if no nonces are available.
pub fn acquire_nonce() -> Option<AcquiredNonce> {
    NONCE_POOL.get()?.acquire()
}

/// Kick off a background refresh for this nonce slot after tx submission.
/// Returns immediately — does NOT block the caller.
pub fn spawn_nonce_refresh(index: usize) {
    if let Some(pool) = NONCE_POOL.get() {
        pool.spawn_refresh(index);
    }
}

/// Release a nonce without refreshing (e.g., if tx was never sent).
pub fn release_nonce(index: usize) {
    if let Some(pool) = NONCE_POOL.get() {
        pool.release(index);
    }
}

// ─── CLI management functions ────────────────────────────────────────

pub async fn nonce_management_menu() {
    loop {
        println!();
        println!("{}", "═══════════════════════════════════════════".cyan());
        println!("  {}", "Advance Nonce Management".cyan().bold());
        println!("{}", "═══════════════════════════════════════════".cyan());
        println!("  {} Create nonce accounts", "[ 1. ]".green());
        println!("  {} View nonce status", "[ 2. ]".green());
        println!("  {} Close nonce accounts (reclaim SOL)", "[ 3. ]".green());
        println!("  {} Back", "[ 0. ]".red());
        println!("{}", "═══════════════════════════════════════════".cyan());
        print!("\n  {} ", "Select option >>".yellow());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "1" => create_nonce_accounts_cli().await,
            "2" => show_nonce_status().await,
            "3" => close_nonce_accounts_cli().await,
            "0" => break,
            _ => println!("{}", "  ⚠ Invalid option.".red()),
        }
    }
}

async fn create_nonce_accounts_cli() {
    print!(
        "\n  {} ",
        "How many nonce accounts to create? >>".yellow()
    );
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let count: usize = match input.trim().parse() {
        Ok(n) if n > 0 && n <= 50 => n,
        _ => {
            println!("{}", "  ⚠ Enter a number between 1 and 50.".red());
            return;
        }
    };

    let total_cost_lamports = count as u64 * NONCE_RENT_LAMPORTS;
    let total_cost_sol = total_cost_lamports as f64 / 1e9;

    println!(
        "\n  {} Creating {} nonce accounts (cost: ~{:.6} SOL for rent)",
        "📝",
        count,
        total_cost_sol
    );

    // Check wallet balance
    match RPC_CLIENT.get_balance(&*SIGNER_PUBKEY).await {
        Ok(balance) => {
            let balance_sol = balance as f64 / 1e9;
            println!("  {} Wallet balance: {:.6} SOL", "💰", balance_sol);
            if balance < total_cost_lamports + 10_000 * count as u64 {
                println!(
                    "{}",
                    "  ⚠ Insufficient balance for nonce account creation.".red()
                );
                return;
            }
        }
        Err(e) => {
            println!("{}", format!("  ⚠ Failed to check balance: {}", e).red());
            return;
        }
    }

    let mut existing = load_nonce_pubkeys();
    let mut created = 0usize;

    for i in 0..count {
        let nonce_keypair = Keypair::new();
        let nonce_pubkey = nonce_keypair.pubkey();

        // create_nonce_account returns 2 instructions:
        // 1. CreateAccount (allocate + fund)
        // 2. InitializeNonceAccount (set authority)
        let create_ixs = system_instruction::create_nonce_account(
            &*SIGNER_PUBKEY,    // payer
            &nonce_pubkey,      // new nonce account
            &*SIGNER_PUBKEY,    // nonce authority
            NONCE_RENT_LAMPORTS,
        );

        let blockhash = match RPC_CLIENT
            .get_latest_blockhash()
            .await
        {
            Ok(h) => h,
            Err(e) => {
                println!(
                    "{}",
                    format!("  ⚠ Failed to get blockhash: {}. Stopping.", e).red()
                );
                break;
            }
        };

        let tx = Transaction::new_signed_with_payer(
            &create_ixs,
            Some(&*SIGNER_PUBKEY),
            &[&SIGNER_KEYPAIR.insecure_clone(), &nonce_keypair],
            blockhash,
        );

        match RPC_CLIENT.send_and_confirm_transaction(&tx).await {
            Ok(sig) => {
                existing.push(nonce_pubkey);
                created += 1;
                println!(
                    "  {} [{}/{}] Created nonce account: {} (sig: {})",
                    "✅".green(),
                    i + 1,
                    count,
                    nonce_pubkey,
                    sig
                );
            }
            Err(e) => {
                println!(
                    "  {} [{}/{}] Failed to create nonce account: {}",
                    "❌".red(),
                    i + 1,
                    count,
                    e
                );
            }
        }
    }

    if created > 0 {
        save_nonce_pubkeys(&existing);
        println!(
            "\n  {} Created {}/{} nonce accounts. Saved to {}",
            "✅".green(),
            created,
            count,
            NONCE_ACCOUNTS_PATH
        );
    }
}

pub async fn show_nonce_status() {
    let pubkeys = load_nonce_pubkeys();
    if pubkeys.is_empty() {
        println!("{}", "\n  ⚠ No nonce accounts found.".yellow());
        return;
    }

    println!(
        "\n  {} Nonce accounts ({} total):",
        "📋",
        pubkeys.len()
    );
    println!("  {}", "─".repeat(80));

    for (i, pk) in pubkeys.iter().enumerate() {
        match RPC_CLIENT.get_account(pk).await {
            Ok(account) => {
                let balance_sol = account.lamports as f64 / 1e9;
                let hash = parse_nonce_hash_from_data(&account.data);
                let hash_str = hash
                    .map(|h| format!("{}", h))
                    .unwrap_or_else(|| "INVALID".red().to_string());
                println!(
                    "  [{}] {} | {:.6} SOL | Nonce: {}",
                    i + 1,
                    pk,
                    balance_sol,
                    &hash_str[..16.min(hash_str.len())]
                );
            }
            Err(_) => {
                println!(
                    "  [{}] {} | {}",
                    i + 1,
                    pk,
                    "UNREACHABLE".red()
                );
            }
        }
    }
    println!("  {}", "─".repeat(80));
}

async fn close_nonce_accounts_cli() {
    let pubkeys = load_nonce_pubkeys();
    if pubkeys.is_empty() {
        println!("{}", "\n  ⚠ No nonce accounts to close.".yellow());
        return;
    }

    println!(
        "\n  {} This will close ALL {} nonce accounts and reclaim SOL.",
        "⚠".yellow(),
        pubkeys.len()
    );
    print!("  {} ", "Type 'yes' to confirm >>".red());
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim() != "yes" {
        println!("  Cancelled.");
        return;
    }

    let mut remaining = Vec::new();

    for pk in &pubkeys {
        // Withdraw all lamports to close the nonce account
        let withdraw_ix = system_instruction::withdraw_nonce_account(
            pk,                 // nonce account
            &*SIGNER_PUBKEY,    // authority
            &*SIGNER_PUBKEY,    // recipient
            NONCE_RENT_LAMPORTS,
        );

        let blockhash = match RPC_CLIENT.get_latest_blockhash().await {
            Ok(h) => h,
            Err(_) => {
                remaining.push(*pk);
                continue;
            }
        };

        let tx = Transaction::new_signed_with_payer(
            &[withdraw_ix],
            Some(&*SIGNER_PUBKEY),
            &[&SIGNER_KEYPAIR.insecure_clone()],
            blockhash,
        );

        match RPC_CLIENT.send_and_confirm_transaction(&tx).await {
            Ok(sig) => {
                println!("  {} Closed {} (sig: {})", "✅".green(), pk, sig);
            }
            Err(e) => {
                println!("  {} Failed to close {}: {}", "❌".red(), pk, e);
                remaining.push(*pk);
            }
        }
    }

    save_nonce_pubkeys(&remaining);
    println!(
        "\n  {} Done. {} accounts remaining.",
        "✅".green(),
        remaining.len()
    );
}
