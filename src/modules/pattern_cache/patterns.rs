use super::pattern_translator::{BuyIxRaw, ManualPatternRaw};

/// Define your manual patterns here. Each entry is translated into a `ManualPattern`
/// at startup and matched against incoming mint transactions.
///
/// Only define fields you want to check — undefined (None) fields are skipped.
///
/// Fields:
///   label:            Optional name for logging
///   dev_cu_price:     "NULL" (== 0), "NOT_NULL" (> 0), or exact number as string
///   dev_cu_limit:     "NULL" (== 0), "NOT_NULL" (> 0), or exact number as string
///   mint_instructions: Ordered instruction names separated by ">>"
///   dev_buy_instruction_data: Buy instruction filter (name + amount condition)
///   stop_loss:        "NULL" (disabled) or percentage as string (e.g. "50")
///   take_profit:      Vec<f64> — TP percentage levels
///   sell_amounts:     Vec<f64> — sell % at each TP level (defaults to equal split)
///   token_version:    "V1" or "V2"
///   alt_addresses:    Vec of base58 ALT pubkey strings
///   mint_tx_version:  "Legacy" or "V0"

pub fn get_raw_manual_patterns() -> Vec<ManualPatternRaw> {
    vec![
        // ── Test Pattern ──
        ManualPatternRaw {
            label: Some("TEST_PATTERN".to_string()),
            dev_cu_price: Some("NULL".to_string()),
            dev_cu_limit: Some("NULL".to_string()),
            mint_instructions: None,
            dev_buy_instruction_data: None,
            stop_loss: None,
            take_profit: vec![400.0],
            sell_amounts: Some(vec![100.0]),
            token_version: None,
            alt_addresses: None,
            mint_tx_version: Some("V0".to_string()),
        },

        // ── PATTERN 1 ──
        ManualPatternRaw {
            label: Some("PATTERN_1".to_string()),
            dev_cu_price: Some("NULL".to_string()),
            dev_cu_limit: Some("NOT_NULL".to_string()),
            mint_instructions: Some("ComputeBudgetProgram::SetComputeUnitLimit>>Pumpfun::Create_v2>>Pumpfun::Extend_account>>AssociatedTokenProgram::CreateIdempotent>>Pumpfun::Buy_exact_sol_in".to_string()),
            dev_buy_instruction_data: Some(BuyIxRaw {
                name: "Buy_exact_sol_in".to_string(),
                amount: "SPENDABLE_SOL_IN>>DEVIDED>>1000000000".to_string(),
            }),
            stop_loss: None,
            take_profit: vec![400.0],
            sell_amounts: Some(vec![100.0]),
            token_version: None,
            alt_addresses: Some(vec!["7mFD2mUtRS65XstiSAvCJuYmdesZoQwCwRJhq1p3eRMe".to_string()]),
            mint_tx_version: Some("V0".to_string()),
        },
    ]
}
