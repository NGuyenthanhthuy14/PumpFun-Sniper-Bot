use pumpfun_sniper::*;

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    nonce_management_menu().await;
}
