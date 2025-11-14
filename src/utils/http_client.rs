use reqwest::Client;
use std::time::Duration;

/// Create a high-performance HTTP client with connection pooling
pub fn create_http_client() -> Client {
    Client::builder()
        .pool_max_idle_per_host(10) // Reuse connections
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .gzip(true) // Enable compression
        .build()
        .expect("Failed to create HTTP client")
}
