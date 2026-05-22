use colored::*;
use pumpfun_sniper::*;

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    println!("{}", "\n  🔄 Wallet Rotation - Cross Chain Swap".yellow().bold());
    println!("{}", "  ─────────────────────────────────────".bright_black());
    match rotate_wallet().await {
        Ok(_) => println!("{}", "\n  ✅ Wallet rotation complete.".green()),
        Err(e) => println!("{}", format!("\n  ❌ Wallet rotation failed: {}", e).red()),
    }
}
