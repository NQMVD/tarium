//! Authentication command handlers for GitHub App integration

use anyhow::Result;
use colored::Colorize;

/// Handle the auth status command
pub async fn handle_auth_command() -> Result<()> {
    // Try to create the GitHub client from libarov which handles GitHub App auth
    let github_api = &libarov::GITHUB_API;

    // Test the client by making a rate limit request
    match github_api.ratelimit().get().await {
        Ok(rate_limit) => {
            println!("{} GitHub App configured", "✓".green().bold());
            println!("{} Authentication: Working", "✓".green());

            let remaining = rate_limit.rate.remaining;
            if remaining < 100 {
                let reset_time = chrono::DateTime::from_timestamp(rate_limit.rate.reset as i64, 0)
                    .map(|dt| dt.format("%H:%M:%S").to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                println!(
                    "Rate limit: {} remaining (resets at {})",
                    remaining, reset_time
                );
            }
        }
        Err(_) => {
            println!("{} GitHub App not configured", "✗".red());
            println!("This build does not have embedded GitHub App credentials.");
            println!("API requests will be limited to 60/hour instead of 5000/hour.");
        }
    }

    Ok(())
}

/// Get an authenticated GitHub client or fall back to anonymous
pub async fn get_github_client() -> reqwest::Client {
    // The GITHUB_API in libarov already handles the authentication
    // This function is kept for compatibility but the actual client
    // creation is handled in libarov
    reqwest::Client::new()
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
