use crate::*;
use chrono::Local;
use std::fs::{File, create_dir_all};
use std::io::{BufWriter, Write};

pub fn check_auto_turn_off_time(_mode: &str) -> bool {
    create_dir_all("src/assets/panel").unwrap_or(());

    let file_name = format!(
        "src/assets/panel/token_monitor_{}.csv",
        Local::now().format("%Y-%m-%d")
    );
    let file = File::create(file_name).expect("Failed to create token database file");

    let mut writer = BufWriter::new(file);

    let headers =
        "No,Mint,Creator,BudgetComputeUnitLimit,BudgetComputeUnitPrice,DEV_BUY_AMOUNT,ATH\n";

    writer
        .write_all(headers.as_bytes())
        .expect("Failed to write CSV headers");
    writer.flush().expect("Failed to flush headers");

    let lists = TOKEN_DB.get_list_all().unwrap();

    let result_string = lists
        .iter()
        .enumerate()
        .map(|(idx, ele)| {
            format!(
                "{},{},{},{},{},{},{}",
                idx + 1,
                ele.0,
                ele.1.token_creator,
                ele.1.mint_budget_compute_unit_limit,
                ele.1.mint_budget_compute_unit_price,
                ele.1.dev_buy_amount_sol,
                ele.1.token_max_price * ele.1.token_total_supply as f64
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    writer.write_all(result_string.as_bytes()).expect("Failed to write CSV file content");
    writer.flush().expect("Failed to flush content");

    // let head_line = format!(
    //     "{:<3} | {:<44} | {:<12} | {:<15} | {:<15} | {:<12} |  {:<12}",
    //     "IDX",
    //     "Mint Addr",
    //     "Price (sol)",
    //     "Max Peak (sol)",
    //     "Buy Point MC",
    //     "TP Status",
    //     "TS Status",
    // );

    // let current = if *SHUT_DOWN_TIMER_SELL_ALL {
    //     let current_time = now.format("%H:%M:%S").to_string();
    //     let comparing_time = format!("{}", *SHUT_DOWN_TIME);
    //     if current_time == comparing_time {
    //         info!("[AUTO TURN OFF]\t\t*SELLING ALL TOKENS ... ");
    //         return true;
    //     };
    //     format!("Shutdown Timer ENABLED : {}", *SHUT_DOWN_TIME)
    // } else {
    //     format!("Shutdown Timer DISABLED")
    // };
    // Format the current timestamp with milliseconds and the sorted result
    // let msg = format!(
    //     "Pump.fun Sniper Bot Overview Panel ( {mode} ) - {}.{:03}  ( ALL {} datas ) {current}\n{}\n{}",
    //     now.format("%Y-%m-%d_%H:%M:%S"), // Format to include hour, minute, and second
    //     now.timestamp_subsec_millis(),   // Milliseconds part
    //     lists.len(),
    //     head_line,
    //     result_string,
    // );

    // Write the message to the file

    false
}
