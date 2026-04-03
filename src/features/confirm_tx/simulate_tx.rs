use crate::*;
use solana_client::rpc_config::{RpcSendTransactionConfig, RpcSimulateTransactionConfig};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, transaction::Transaction,
};
use std::time::Instant;

const DEFAULT_SELL_CU_LIMIT: u32 = 200_000;

pub async fn simulate_and_send_sell_tx(
    raw_instructions: Vec<Instruction>,
    tag: String,
) -> Option<String> {
    // Build a simulation transaction without compute budget instructions
    let sim_tx = Transaction::new_with_payer(&raw_instructions, Some(&SIGNER_PUBKEY));

    let sim_config = RpcSimulateTransactionConfig {
        sig_verify: false,
        replace_recent_blockhash: true,
        commitment: Some(CommitmentConfig::confirmed()),
        ..Default::default()
    };

    let sim_result = RPC_CLIENT
        .simulate_transaction_with_config(&sim_tx, sim_config)
        .await;

    let cu_limit = match sim_result {
        Ok(response) => {
            if let Some(ref err) = response.value.err {
                error!(
                    "[SIMULATE]\n\t* Simulation failed: {:?}\n\t* Falling back to default CU\n\t* {}",
                    err,
                    tag.clone()
                );
                DEFAULT_SELL_CU_LIMIT
            } else {
                let units = response
                    .value
                    .units_consumed
                    .unwrap_or(DEFAULT_SELL_CU_LIMIT as u64);
                (units + 1000) as u32
            }
        }
        Err(e) => {
            error!(
                "[SIMULATE]\n\t* RPC simulation error: {:?}\n\t* Falling back to default CU\n\t* {}",
                e,
                tag.clone()
            );
            DEFAULT_SELL_CU_LIMIT
        }
    };

    // Acquire a nonce for the actual send
    let nonce = match acquire_nonce() {
        Some(n) => n,
        None => {
            error!("[SELL] No nonce account available\n\t* {}", tag);
            return None;
        }
    };

    // Build the final transaction with compute budget
    let mut total_instruction = Vec::new();
    // Advance nonce MUST be the first instruction
    total_instruction.push(nonce.advance_ix);
    total_instruction.push(ComputeBudgetInstruction::set_compute_unit_limit(cu_limit));
    total_instruction.push(ComputeBudgetInstruction::set_compute_unit_price(
        *SELL_MICRO_LAMPORTS as u64,
    ));
    total_instruction.extend(raw_instructions);

    let mut transaction = Transaction::new_with_payer(&total_instruction, Some(&SIGNER_PUBKEY));
    transaction
        .try_sign(&[SIGNER_KEYPAIR.insecure_clone()], nonce.nonce_hash)
        .expect("Failed to sign sell transaction");

    let tx_submission_start = Instant::now();

    let result = match RPC_CLIENT
        .send_transaction_with_config(
            &transaction,
            RpcSendTransactionConfig {
                skip_preflight: true,
                ..Default::default()
            },
        )
        .await
    {
        Ok(signature) => {
            // TX was submitted — nonce will be consumed on-chain
            spawn_nonce_refresh(nonce.index);
            println!(
                "Transaction(sell RPC) submission took: {:?}",
                tx_submission_start.elapsed()
            );
            info!(
                "[SUBMIT]\n\t* Service: RPC\n\t* Hash: {:?}\n\t* CU: {}\n\t* {}",
                signature, cu_limit, tag
            );
            Some(signature.to_string())
        }
        Err(e) => {
            // RPC error — tx was never sent, nonce not consumed
            release_nonce(nonce.index);
            error!("[SUBMIT]\n\t* RPC send error: {:?}\n\t* {}", e, tag);
            None
        }
    };

    result
}
