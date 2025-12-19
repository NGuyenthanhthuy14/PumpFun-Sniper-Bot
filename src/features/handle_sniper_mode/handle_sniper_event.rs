use crate::*;
// use colored::*;
use dashmap::DashMap;
use solana_sdk::pubkey::Pubkey;

pub async fn handle_sniper_event(
    trade_data: (
        bool,
        Vec<MintEvent>,
        Vec<BuyEvent>,
        Vec<SellEvent>,
        Vec<MintInstructionAccounts>,
        Vec<BuyInstructionAccounts>,
        Vec<SellInstructionAccounts>,
    ),
    budget_compute_data: (u32, u64),
    tx_id: String,
) -> DashMap<Pubkey, TokenDatabaseSchema> {
    let (
        is_bundler_buy,
        mint_events,
        buy_events,
        sell_events,
        mint_ixs_accounts,
        _buy_ixs_accounts,
        _sell_ixs_accounts,
    ) = trade_data;

    let (unit, price) = budget_compute_data;

    let return_data: DashMap<Pubkey, TokenDatabaseSchema> = DashMap::new();

    for (i, mint_event) in mint_events.iter().enumerate() {
        let mint_ix_accounts = &mint_ixs_accounts[i];
        match (unit, price) {
            (_, 6666666) => {
                println!(
                    "Launched filtered token,\t*unit: {}\t*price: {}",
                    unit, price
                );
                let token_data: TokenDatabaseSchema = TokenDatabaseSchema::new_from_mint(
                    mint_event.clone(),
                    mint_ix_accounts.clone(),
                    (unit, price),
                    tx_id.clone(),
                );

                //Time based buying logic after bundle finished
                let token_data_clone = token_data.clone();
                tokio::spawn(async move {
                    let _ = proceed_time_based_buying_logic(token_data_clone).await;
                });

                return_data.insert(token_data.token_mint, token_data);
            }
            _ => {}
        }
    }

    for (_i, buy_event) in buy_events.iter().enumerate() {
        if let Some(token_data) = TOKEN_DB.get(buy_event.mint).unwrap() {
            let updated_token_data: TokenDatabaseSchema = update_status_from_buy_event(
                token_data.clone(),
                buy_event.clone(),
                is_bundler_buy,
                tx_id.to_string(),
            );
            return_data.insert(updated_token_data.token_mint, updated_token_data);
        }
    }

    for (_i, sell_event) in sell_events.iter().enumerate() {
        if let Some(token_data) = TOKEN_DB.get(sell_event.mint).unwrap() {
            if let Some(updated_token_data) = update_status_from_sell_event(
                token_data.clone(),
                sell_event.clone(),
                tx_id.to_string(),
            ) {
                return_data.insert(updated_token_data.token_mint, updated_token_data);
            }
        }
    }
    return_data
}
