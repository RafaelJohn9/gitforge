use std::time::Duration;

use anyhow::anyhow;
use reqwest::blocking::Client;

pub struct Fetcher {
    client: Client,
}

impl Fetcher {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("gitforge-fetcher")
                .build()
                .unwrap(),
        }
    }

    /// Fetch raw content from a URL
    pub fn fetch_content(&self, url: &str) -> anyhow::Result<String> {
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch from {}: {}", url, e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Request failed with status {}: {}",
                response.status(),
                url
            ));
        }

        response
            .text()
            .map_err(|e| anyhow!("Failed to read response: {}", e))
    }

    /// Fetch and parse JSON from a URL
    pub fn fetch_json(&self, url: &str) -> anyhow::Result<serde_json::Value> {
        let response = self
            .client
            .get(url)
            .send()
            .map_err(|e| anyhow!("Failed to fetch JSON from {}: {}", url, e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "JSON request failed with status {}: {}",
                response.status(),
                url
            ));
        }

        response
            .json()
            .map_err(|e| anyhow!("Failed to parse JSON: {}", e))
    }
}
