use reqwest::{Client, ClientBuilder};
use std::time::Duration;

/// Create a configured HTTP client for making requests to git platform APIs
pub fn create_http_client() -> Client {
    ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .user_agent("hgitmap/0.1.0")
        .build()
        .expect("Failed to create HTTP client")
}

/// Create an HTTP client that accepts self-signed certificates (development only)
#[cfg(debug_assertions)]
pub fn create_http_client_allow_self_signed() -> Client {
    ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .user_agent("hgitmap/0.1.0")
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to create HTTP client")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_http_client() {
        let client = create_http_client();
        assert!(client.timeout().is_some());
    }
}
