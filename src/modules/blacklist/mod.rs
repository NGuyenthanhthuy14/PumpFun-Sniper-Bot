use colored::Colorize;

use crate::*;

pub fn check_blacklisted(token_data: &TokenDatabaseSchema) -> bool {
    if WALLET_BLACKLIST_PATH.contains(&token_data.token_creator.to_string()) {
        warning!("Token creator is blacklisted wallet: {}", &token_data.pump_fun_swap_accounts.creator_vault.to_string().red());
        true
    } else if TOKEN_BLACKLIST_PATH
        .contains(&token_data.token_mint.to_string())
    {
        warning!("Token is blacklisted token: {}", &token_data.token_mint.to_string().red());
        true
    } else {
        false
    }
}
