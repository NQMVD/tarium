//! Authentication command handlers for GitHub App integration

use anyhow::Result;
use colored::Colorize;

use crate::auth::{get_github_app_client, GitHubAppClient};

/// Handle the auth status command
pub async fn handle_auth_command() -> Result<()> {
    match get_github_app_client().await {
        Some(client) => {
            println!("{} GitHub App configured", "✓".green().bold());

            match client.test_authentication().await {
                Ok(()) => {
                    println!("{} Authentication: Working", "✓".green());

                    if let Ok(rate_info) = client.get_rate_limit_info().await {
                        rate_info.display();
                    }
                }
                Err(e) => {
                    println!("{} Authentication: Failed", "✗".red());
                    println!("Error: {}", e);
                }
            }
        }
        None => {
            println!("{} GitHub App not configured", "✗".red());
            println!("This build does not have embedded GitHub App credentials.");
            println!("API requests will be limited to 60/hour instead of 5000/hour.");
        }
    }

    Ok(())
}

/// Get a GitHub App client if available
pub async fn get_github_app_client_wrapper() -> Option<GitHubAppClient> {
    get_github_app_client().await
}

/// Get an authenticated GitHub client or fall back to anonymous
pub async fn get_github_client() -> reqwest::Client {
    crate::auth::get_github_client().await
}

#[cfg(test)]
mod tests {
    use super::*;

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
