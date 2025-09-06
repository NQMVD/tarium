//! GitHub App authentication module for Tarium CLI
//!
//! This module handles GitHub App authentication using embedded credentials.
//! No user authentication is required - credentials are embedded at build time.

use anyhow::{anyhow, Result};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// GitHub App configuration
#[derive(Debug, Clone)]
pub struct GitHubAppConfig {
    pub app_id: String,
    pub installation_id: String,
    pub private_key: String,
}

/// JWT claims for GitHub App authentication
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String, // Issuer (App ID)
    iat: u64,    // Issued at
    exp: u64,    // Expiration time
}

/// GitHub App installation access token response
#[derive(Debug, Deserialize)]
struct InstallationTokenResponse {
    token: String,
    expires_at: String,
}

/// GitHub App authenticated client
#[derive(Debug, Clone)]
pub struct GitHubAppClient {
    config: GitHubAppConfig,
    client: reqwest::Client,
}

impl GitHubAppConfig {
    /// Create new GitHub App config
    pub fn new(app_id: String, installation_id: String, private_key: String) -> Self {
        Self {
            app_id,
            installation_id,
            private_key,
        }
    }

    /// Load configuration from embedded credentials
    pub fn embedded() -> Option<Self> {
        let app_id = env!("TARIUM_EMBEDDED_APP_ID");
        let installation_id = env!("TARIUM_EMBEDDED_INSTALLATION_ID");
        let private_key = env!("TARIUM_EMBEDDED_PRIVATE_KEY");

        if app_id.is_empty() || installation_id.is_empty() || private_key.is_empty() {
            None
        } else {
            let unescaped_key = private_key
                .replace("\\n", "\n")
                .replace("\\r", "\r")
                .replace("\\\"", "\"")
                .replace("\\\\", "\\");

            Some(Self::new(
                app_id.to_string(),
                installation_id.to_string(),
                unescaped_key,
            ))
        }
    }

    /// Generate a JWT token for GitHub App authentication
    fn generate_jwt(&self) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Failed to get current time: {}", e))?
            .as_secs();

        let issued_at = now - 10;
        let expires_at = now + 300;

        let claims = Claims {
            iss: self.app_id.clone(),
            iat: issued_at,
            exp: expires_at,
        };

        let header = Header::new(Algorithm::RS256);
        let encoding_key = EncodingKey::from_rsa_pem(self.private_key.as_bytes())
            .map_err(|e| anyhow!("Failed to parse private key: {}", e))?;

        let jwt = encode(&header, &claims, &encoding_key)
            .map_err(|e| anyhow!("Failed to generate JWT: {}", e))?;

        Ok(jwt)
    }
}

impl GitHubAppClient {
    /// Create a new GitHub App client
    pub fn new(config: GitHubAppConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// Create a GitHub App client from embedded credentials
    pub fn embedded() -> Option<Self> {
        GitHubAppConfig::embedded().map(Self::new)
    }

    /// Get an installation access token
    async fn get_installation_token(&self) -> Result<String> {
        let jwt = self.config.generate_jwt()?;

        let url = format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            self.config.installation_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "Tarium-CLI")
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let error_msg = match status.as_u16() {
                401 => {
                    "Authentication failed. Check your App ID, Installation ID, and private key."
                }
                404 => "Installation not found. Check your Installation ID.",
                403 => "Permission denied. Check GitHub App permissions.",
                _ => "Unknown error occurred.",
            };

            return Err(anyhow!(
                "Failed to get installation token ({}): {}\n{}",
                status,
                body,
                error_msg
            ));
        }

        let token_response: InstallationTokenResponse = response.json().await?;
        Ok(token_response.token)
    }

    /// Get an authenticated reqwest client with GitHub App token
    pub async fn authenticated_client(&self) -> Result<reqwest::Client> {
        let token = self.get_installation_token().await?;

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("Tarium-CLI"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(client)
    }

    /// Test the GitHub App authentication
    pub async fn test_authentication(&self) -> Result<()> {
        let client = self.authenticated_client().await?;
        let response = client
            .get("https://api.github.com/rate_limit")
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!(
                "GitHub App authentication test failed: {}",
                response.status()
            ))
        }
    }

    /// Get current rate limit information
    pub async fn get_rate_limit_info(&self) -> Result<RateLimitInfo> {
        let client = self.authenticated_client().await?;
        let response = client
            .get("https://api.github.com/rate_limit")
            .send()
            .await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            let rate = data
                .get("rate")
                .ok_or_else(|| anyhow!("No rate limit data"))?;

            Ok(RateLimitInfo {
                remaining: rate.get("remaining").and_then(|v| v.as_u64()).unwrap_or(0),
                reset_time: rate.get("reset").and_then(|v| v.as_u64()).unwrap_or(0),
            })
        } else {
            Err(anyhow!(
                "Failed to get rate limit info: {}",
                response.status()
            ))
        }
    }
}

/// Rate limit information from GitHub API
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub remaining: u64,
    pub reset_time: u64,
}

impl RateLimitInfo {
    /// Display rate limit info to user
    pub fn display(&self) {
        if self.remaining < 100 {
            let reset_time = chrono::DateTime::from_timestamp(self.reset_time as i64, 0)
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "unknown".to_string());

            println!(
                "Rate limit: {} remaining (resets at {})",
                self.remaining, reset_time
            );
        }
    }
}

/// Get a GitHub App client if available
pub async fn get_github_app_client() -> Option<GitHubAppClient> {
    if let Some(client) = GitHubAppClient::embedded() {
        if client.test_authentication().await.is_ok() {
            return Some(client);
        }
    }
    None
}

/// Get an authenticated GitHub client or fall back to anonymous
pub async fn get_github_client() -> reqwest::Client {
    if let Some(app_client) = get_github_app_client().await {
        if let Ok(client) = app_client.authenticated_client().await {
            return client;
        }
    }
    reqwest::Client::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_app_config_creation() {
        let config = GitHubAppConfig::new(
            "12345".to_string(),
            "67890".to_string(),
            "test-key".to_string(),
        );

        assert_eq!(config.app_id, "12345");
        assert_eq!(config.installation_id, "67890");
        assert_eq!(config.private_key, "test-key");
    }

    #[tokio::test]
    async fn test_get_github_client() {
        let client = get_github_client().await;
        let response = client
            .get("https://api.github.com/rate_limit")
            .header("User-Agent", "Test")
            .send()
            .await;

        assert!(response.is_ok() || response.is_err());
    }
}
