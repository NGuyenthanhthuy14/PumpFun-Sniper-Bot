use crate::*;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

pub fn get_http_client() -> Arc<Client> {
    HTTP_CLIENT.clone()
}

pub async fn pre_warm_zero_slot_endpoint(client: Arc<Client>) {
    println!("🔥 Pre-warming 0-slot endpoint...");

    for attempt in 1..=3 {
        match client.get(&*ZERO_SLOT_ENDPOINT).send().await {
            Ok(response) => {
                println!(
                    "✅ 0-slot endpoint ready (attempt {}): HTTP {}",
                    attempt,
                    response.status()
                );

                if response.status().is_success() {
                    println!("🎯 Successfully connected to 0-slot service");
                }
                break;
            }
            Err(e) if attempt < 3 => {
                println!("⚠️ 0-slot warm-up attempt {} failed: {:?}", attempt, e);
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
            }
            Err(e) => {
                eprintln!("❌ Failed to pre-warm 0-slot endpoint: {:?}", e);
            }
        }
    }
}

pub async fn pre_warm_helius_endpoint(client: Arc<Client>) {
    println!("🔥 Pre-warming Helius endpoint...");

    for attempt in 1..=3 {
        match client.get(&*HELIUS_ENDPOINT).send().await {
            Ok(response) => {
                println!(
                    "✅ Helius endpoint ready (attempt {}): HTTP {}",
                    attempt,
                    response.status()
                );

                if response.status().is_success() {
                    println!("🎯 Successfully connected to Helius service");
                }
                break;
            }
            Err(e) if attempt < 3 => {
                println!("⚠️ Helius warm-up attempt {} failed: {:?}", attempt, e);
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
            }
            Err(e) => {
                eprintln!("❌ Failed to pre-warm Helius endpoint: {:?}", e);
            }
        }
    }
}
