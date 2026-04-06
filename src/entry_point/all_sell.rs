use colored::*;

#[tokio::main]
async fn main() {
    println!("{}", "\n  🔄 Executing All Sell...".yellow());
    println!(
        "{}",
        "  ⚠️  All Sell module has unresolved dependencies. Fix all_sell/mod.rs first.".red()
    );
}
