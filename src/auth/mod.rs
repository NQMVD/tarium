//! GitHub App authentication module for Tarium CLI
//!
//! This module handles GitHub App authentication which allows the application
//! to access GitHub APIs without requiring users to authenticate individually.

pub mod storage;

pub use storage::{AppCredentials, CredentialStatus, CredentialStorage};

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

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let app_id = std::env::var("TARIUM_GITHUB_APP_ID")
            .map_err(|_| anyhow!("TARIUM_GITHUB_APP_ID environment variable not set"))?;

        let installation_id = std::env::var("TARIUM_GITHUB_INSTALLATION_ID")
            .map_err(|_| anyhow!("TARIUM_GITHUB_INSTALLATION_ID environment variable not set"))?;

        let private_key = if let Ok(key_path) = std::env::var("TARIUM_GITHUB_PRIVATE_KEY_PATH") {
            std::fs::read_to_string(&key_path)
                .map_err(|e| anyhow!("Failed to read private key from {}: {}", key_path, e))?
        } else if let Ok(key_content) = std::env::var("TARIUM_GITHUB_PRIVATE_KEY") {
            key_content
        } else {
            return Err(anyhow!(
                "Either TARIUM_GITHUB_PRIVATE_KEY_PATH or TARIUM_GITHUB_PRIVATE_KEY must be set"
            ));
        };

        Ok(Self::new(app_id, installation_id, private_key))
    }

    /// Load configuration from embedded credentials (for distribution)
    pub fn embedded() -> Option<Self> {
        // Check if we have embedded credentials from build.rs
        let app_id = env!("TARIUM_EMBEDDED_APP_ID");
        let installation_id = env!("TARIUM_EMBEDDED_INSTALLATION_ID");
        let private_key = env!("TARIUM_EMBEDDED_PRIVATE_KEY");

        // If any of the embedded values are empty, return None
        if app_id.is_empty() || installation_id.is_empty() || private_key.is_empty() {
            None
        } else {
            // Unescape the private key that was escaped during build
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

        // GitHub requires JWT expiration to be no more than 10 minutes in the future
        // and issued time should not be more than 60 seconds in the past
        let issued_at = now - 10; // Issued 10 seconds ago to account for clock skew
        let expires_at = now + 300; // Expires in 5 minutes (well under GitHub's 10 minute limit)

        log::debug!("Generating JWT for App ID: {}", self.app_id);
        log::debug!("Current timestamp: {}", now);
        log::debug!("JWT issued at: {}", issued_at);
        log::debug!("JWT expires at: {}", expires_at);
        log::debug!("JWT lifetime: {} seconds", expires_at - issued_at);

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

        log::debug!(
            "Generated JWT successfully, length: {} characters",
            jwt.len()
        );
        Ok(jwt)
    }
}

impl GitHubAppClient {
    /// Create a new GitHub App client
    pub fn new(config: GitHubAppConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// Create a GitHub App client from environment variables
    pub fn from_env() -> Result<Self> {
        let config = GitHubAppConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Create a GitHub App client from embedded credentials
    pub fn embedded() -> Option<Self> {
        GitHubAppConfig::embedded().map(Self::new)
    }

    /// Try to create a GitHub App client from various sources
    pub fn auto() -> Result<Self> {
        // Try embedded credentials first (for distribution)
        if let Some(client) = Self::embedded() {
            log::info!("Using embedded GitHub App credentials");
            return Ok(client);
        }

        // Try environment variables
        if let Ok(client) = Self::from_env() {
            log::info!("Using GitHub App credentials from environment");
            return Ok(client);
        }

        // Try stored credentials
        if let Ok(storage) = CredentialStorage::new() {
            if let Ok(Some(creds)) = storage.load_credentials() {
                let config =
                    GitHubAppConfig::new(creds.app_id, creds.installation_id, creds.private_key);
                log::info!("Using stored GitHub App credentials");
                return Ok(Self::new(config));
            }
        }

        Err(anyhow!("No GitHub App credentials found. Set environment variables or use embedded credentials."))
    }

    /// Get an installation access token
    async fn get_installation_token(&self) -> Result<String> {
        let jwt = self.config.generate_jwt()?;

        let url = format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            self.config.installation_id
        );

        log::debug!("Requesting installation token from: {}", url);
        log::debug!("App ID: {}", self.config.app_id);
        log::debug!("Installation ID: {}", self.config.installation_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", jwt))
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "Tarium-CLI")
            .send()
            .await?;

        let status = response.status();
        log::debug!("Response status: {}", status);

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            log::error!("GitHub API error response: {}", body);

            // Provide more helpful error messages based on status code
            let error_msg = match status.as_u16() {
                401 => {
                    if body.contains("exp") && body.contains("too far in the future") {
                        "JWT expiration time is too far in the future. This usually indicates a clock synchronization issue."
                    } else if body.contains("iat") {
                        "JWT issued time is invalid. Check your system clock."
                    } else {
                        "Authentication failed. Check your App ID, Installation ID, and private key."
                    }
                }
                404 => "Installation not found. Check your Installation ID or ensure the GitHub App is installed.",
                403 => "Permission denied. Check that your GitHub App has the required permissions.",
                _ => "Unknown error occurred."
            };

            return Err(anyhow!(
                "Failed to get installation token ({}): {}\nHelp: {}",
                status,
                body,
                error_msg
            ));
        }

        let token_response: InstallationTokenResponse = response.json().await?;
        log::debug!("Successfully obtained installation token");
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
            let rate_limit: serde_json::Value = response.json().await?;

            if let Some(limit) = rate_limit.get("rate").and_then(|r| r.get("limit")) {
                log::info!(
                    "GitHub App authentication successful. Rate limit: {}",
                    limit
                );
            }

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
                limit: rate.get("limit").and_then(|v| v.as_u64()).unwrap_or(0),
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
    pub limit: u64,
    pub remaining: u64,
    pub reset_time: u64,
}

impl RateLimitInfo {
    /// Get a human-readable description of the rate limit status
    pub fn status_description(&self) -> String {
        if self.remaining > 1000 {
            format!(
                "{}/{} API calls remaining (plenty)",
                self.remaining, self.limit
            )
        } else if self.remaining > 100 {
            format!(
                "{}/{} API calls remaining (good)",
                self.remaining, self.limit
            )
        } else if self.remaining > 10 {
            format!(
                "{}/{} API calls remaining (low)",
                self.remaining, self.limit
            )
        } else {
            let reset_time = chrono::DateTime::from_timestamp(self.reset_time as i64, 0)
                .map(|dt| dt.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "unknown".to_string());
            format!(
                "{}/{} API calls remaining (critical - resets at {})",
                self.remaining, self.limit, reset_time
            )
        }
    }
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

    #[test]
    fn test_rate_limit_info_status() {
        let info = RateLimitInfo {
            limit: 5000,
            remaining: 4500,
            reset_time: 1234567890,
        };

        let status = info.status_description();
        assert!(status.contains("plenty"));
    }

    #[tokio::test]
    async fn test_auto_client_creation() {
        // This test will fail without proper credentials, which is expected
        let result = GitHubAppClient::auto();
        // We can't assert success since we don't have test credentials
        // But we can ensure the error is reasonable
        if let Err(e) = result {
            assert!(e.to_string().contains("No GitHub App credentials found"));
        }
    }
}
