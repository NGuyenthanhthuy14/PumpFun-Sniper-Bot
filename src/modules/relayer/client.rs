use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use once_cell::sync::Lazy;

//HTTP endpoint
pub static HTTP_CLIENT: Lazy<Arc<Client>> = Lazy::new(|| {
    println!("🔄 Initializing HTTP client...");

    let _ = rustls::crypto::ring::default_provider().install_default();

    let client = Client::builder()
        .pool_idle_timeout(Duration::from_secs(300))
        .pool_max_idle_per_host(5)
        .tcp_keepalive(Duration::from_secs(10))
        .tcp_nodelay(true)
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .http2_keep_alive_interval(Duration::from_secs(20))
        .http2_keep_alive_timeout(Duration::from_secs(90))
        .http2_keep_alive_while_idle(true)
        .use_rustls_tls()
        .build()
        .expect("Failed to build HTTP client");

    Arc::new(client)
});