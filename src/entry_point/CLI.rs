use colored::*;
use pumpfun_sniper::*;
use std::io::{self, Write};
use yellowstone_grpc_proto::geyser::SubscribeRequestFilterTransactions;

const PATTERN_SERVER_PORT: u16 = 3355;

fn print_banner() {
    println!();
    println!("{}", "    ██████╗ ██╗   ██╗███╗   ███╗██████╗ ███████╗██╗   ██╗███╗   ██╗".cyan().bold());
    println!("{}", "    ██╔══██╗██║   ██║████╗ ████║██╔══██╗██╔════╝██║   ██║████╗  ██║".cyan().bold());
    println!("{}", "    ██████╔╝██║   ██║██╔████╔██║██████╔╝█████╗  ██║   ██║██╔██╗ ██║".cyan().bold());
    println!("{}", "    ██╔═══╝ ██║   ██║██║╚██╔╝██║██╔═══╝ ██╔══╝  ██║   ██║██║╚██╗██║".cyan().bold());
    println!("{}", "    ██║     ╚██████╔╝██║ ╚═╝ ██║██║     ██║     ╚██████╔╝██║ ╚████║".cyan().bold());
    println!("{}", "    ╚═╝      ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚═╝      ╚═════╝ ╚═╝  ╚═══╝".cyan().bold());
    println!();
    println!("{}", "    ███████╗███╗   ██╗██╗██████╗ ███████╗██████╗ ".yellow().bold());
    println!("{}", "    ██╔════╝████╗  ██║██║██╔══██╗██╔════╝██╔══██╗".yellow().bold());
    println!("{}", "    ███████╗██╔██╗ ██║██║██████╔╝█████╗  ██████╔╝".yellow().bold());
    println!("{}", "    ╚════██║██║╚██╗██║██║██╔═══╝ ██╔══╝  ██╔══██╗".yellow().bold());
    println!("{}", "    ███████║██║ ╚████║██║██║     ███████╗██║  ██║".yellow().bold());
    println!("{}", "    ╚══════╝╚═╝  ╚═══╝╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝".yellow().bold());
    println!();
    println!("{}", "    ┌─────────────────────────────────────────────────────────┐".bright_black());
    println!("{}", "    │    ⚡  Pumpfun Sniper Bot  ·  Durable Nonce Engine  ⚡   │".white().bold());
    println!("{}", "    └─────────────────────────────────────────────────────────┘".bright_black());
}

fn menu_row(key: &str, icon: &str, label: &str, is_exit: bool) -> String {
    let key_colored = if is_exit {
        format!(" {} ", key).red().bold().to_string()
    } else {
        format!(" {} ", key).green().bold().to_string()
    };
    let label_colored = if is_exit {
        format!("{}  {}", icon, label).bright_black().bold().to_string()
    } else {
        format!("{}  {}", icon, label).white().bold().to_string()
    };
    format!("      {}  {}", key_colored, label_colored)
}

fn print_menu() {
    let w = 57;
    let bar = "═".repeat(w);

    println!();
    println!("    {}", bar.cyan());
    println!(
        "{}",
        "              M A I N   M E N U".cyan().bold()
    );
    println!("    {}", bar.cyan());
    println!();
    println!("[{}]", menu_row("1.", "🎯", "Start Sniper Bot", false));
    println!();
    println!("[{}]", menu_row("2.", "🔑", "Advance Nonce Management", false));
    println!();
    println!("[{}]", menu_row("3.", "💰", "All Sell", false));
    println!();
    println!("[{}]", menu_row("4.", "🔄", "Wallet Rotation", false));
    println!();
    println!("    {}", bar.cyan());
    println!();
    println!("[{}]", menu_row("0.", "⚓", "Exit", true));
    println!();
    println!("    {}", bar.cyan());
    println!();
    print!("    {} ", "▶  Select option >>".yellow().bold());
    io::stdout().flush().unwrap();
}

fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

async fn run_sniper_bot() {
    info!("{}", SNIPER_MODE_STR.green());
    let client = get_http_client();

    match LANDING_SERVICE.as_str() {
        "ZERO_SLOT" => {
            pre_warm_zero_slot_endpoint(client).await;
        }
        "HELIUS" => {
            pre_warm_helius_endpoint(client).await;
        }
        _ => {
            println!("Unsupported landing service, defaulting to 0-slot");
            pre_warm_zero_slot_endpoint(client).await;
        }
    }

    // Initialize nonce pool for durable transactions
    init_nonce_pool().await;

    tokio::spawn(async {
        run_pattern_server(PATTERN_SERVER_PORT).await;
    });

    let grpc_config = GrpcClientConfig::new(
        "sniper_mode".to_string(),
        GRPC_ENDPOINT.to_string(),
        GRPC_TOKEN.to_string(),
    );

    let subscribe_pumpfun_program_id = SubscribeRequestFilterTransactions {
        account_include: vec![
            PUMPFUN_PROGRAM_ID.to_string(),
            PUMPSWAP_PROGRAM_ID.to_string(),
        ],
        account_exclude: vec![],
        account_required: vec![],
        vote: Some(false),
        failed: Some(false),
        signature: None,
    };

    if let Err(e) = grpc_config
        .subscribe_with_reconnect(subscribe_pumpfun_program_id)
        .await
    {
        error!(
            "Failed to maintain GRPC connection after all retries: {:?}",
            e
        );
    }
}

async fn run_all_sell() {
    println!("{}", "\n  🔄 Executing All Sell...".yellow());
    println!("{}", "  ⚠️  All Sell module has unresolved dependencies. Fix all_sell/mod.rs first.".red());
}

async fn run_advance_nonce() {
    nonce_management_menu().await;
}

async fn run_wallet_rotation() {
    println!("{}", "\n  🔄 Wallet Rotation - Coming soon".yellow());
}

#[tokio::main]
pub async fn main() {
    print_banner();

    loop {
        print_menu();
        let input = read_input();

        match input.as_str() {
            "1" => {
                run_sniper_bot().await;
                break;
            }
            "2" => {
                run_advance_nonce().await;
            }
            "3" => {
                run_all_sell().await;
            }
            "4" => {
                run_wallet_rotation().await;
            }
            "0" => {
                println!("{}", "\n  👋 Exiting...".cyan());
                break;
            }
            _ => {
                println!("{}", "\n  ⚠️  Invalid option. Try again.".red());
            }
        }
    }
}
