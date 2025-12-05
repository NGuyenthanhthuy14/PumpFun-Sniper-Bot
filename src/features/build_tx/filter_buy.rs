use crate::*;
use colored::*;
use magic_crypt::generic_array::typenum::Pow;

pub fn buy_filter_check(token_data: TokenDatabaseSchema, mode: String) -> bool {
    let mut market_cap_valid = true;
    let mut volume_valid = true;

    if mode == "Sniper_Mode".to_string() {
        if *MARKET_CAP_FILTER {
            if token_data.token_marketcap < *MIN_MARKET_CAP_LIMIT_SOL as f64 {
                market_cap_valid = false;
            }
        }

        if *VOLUME_FILTER {
            if let Some(val) = token_data.token_volume {
                if val < *MIN_VOLUME_LIMIT_SOL as f64 {
                    volume_valid = false;
                }
            }
        }
    } else {
        if *MARKET_CAP_FILTER {
            if token_data.token_marketcap < *MIN_MARKET_CAP_LIMIT_SOL as f64 {
                market_cap_valid = false;
            }
        }
    }

    market_cap_valid && volume_valid
}

pub async fn max_token_holder_check(token_data: TokenDatabaseSchema, mode: String) -> bool {
    let mut max_token_holder_valid = true;

    if *MAX_TOKEN_HOLDER_FILTER {
        if mode != "Sniper_Mode" {
            let data = match RPC_CLIENT
                .get_token_largest_accounts(&token_data.token_mint)
                .await
            {
                Ok(data) => data,
                Err(_) => vec![],
            };

            if let Some(first) = data.get(0) {
                if first.address
                    == token_data
                        .pump_fun_swap_accounts
                        .associated_bonding_curve
                        .to_string()
                {
                    if let Some(second) = data.get(1) {
                        if let Some(val) = second.amount.ui_amount {
                            println!("Max holder amount (second): {}", val);
                            if val > *MAX_TOKEN_HOLDER_LIMIT as f64 {
                                error!(
                                    "[FILTER] => MINT : {}\t* MAX HOLDING {:?} LIMIT {}",
                                    token_data.token_mint, val, *MAX_TOKEN_HOLDER_LIMIT
                                );
                                max_token_holder_valid = false;
                            }
                        }
                    }
                } else {
                    if let Some(val) = first.amount.ui_amount {
                        println!("Max holder amount (first): {}", val);
                        if val > *MAX_TOKEN_HOLDER_LIMIT as f64 {
                            error!(
                                "[FILTER] => MINT : {}\t* MAX HOLDING {:?} LIMIT {}",
                                token_data.token_mint, val, *MAX_TOKEN_HOLDER_LIMIT
                            );
                            max_token_holder_valid = false;
                        }
                    }
                }
            }
        } else {
            for holder in token_data.token_holders.iter() {
                if *holder.1 >= *MAX_TOKEN_HOLDER_LIMIT * 1000000 {
                    error!(
                        "[FILTER] => MINT : {}\t* MAX HOLDING {:?} LIMIT {}",
                        token_data.token_mint, *holder.1 as f64/ 1000000.0, *MAX_TOKEN_HOLDER_LIMIT
                    );
                    max_token_holder_valid = false;
                    break;
                }
            }
        }
    }

    if !max_token_holder_valid {
        let _ = TOKEN_DB.delete(token_data.token_mint);
    }

    max_token_holder_valid
}
