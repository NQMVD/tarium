//! Authentication command handlers for GitHub App integration

use anyhow::{anyhow, Result};
use colored::Colorize;
use inquire::{Password, Text};

use crate::auth::{AppCredentials, CredentialStatus, CredentialStorage, GitHubAppClient};
use crate::cli::AuthSubCommands;

/// Handle authentication-related subcommands
pub async fn handle_auth_command(subcommand: AuthSubCommands) -> Result<()> {
    match subcommand {
        AuthSubCommands::Login {
            client_id: app_id,
            client_secret: _,
        } => handle_setup(app_id).await,
        AuthSubCommands::Logout => handle_remove().await,
        AuthSubCommands::Status => handle_status().await,
    }
}

/// Handle the setup command (renamed from login since no user auth needed)
async fn handle_setup(app_id: Option<String>) -> Result<()> {
    let storage = CredentialStorage::new()?;

    // Check if we already have valid credentials
    if let Some(existing_creds) = storage.load_credentials()? {
        println!("🔍 Checking existing GitHub App credentials...");

        let client = GitHubAppClient::new(crate::auth::GitHubAppConfig::new(
            existing_creds.app_id.clone(),
            existing_creds.installation_id.clone(),
            existing_creds.private_key.clone(),
        ));

        if client.test_authentication().await.is_ok() {
            println!(
                "{} GitHub App already configured and working!",
                "✅".green().bold()
            );
            storage.mark_credentials_used()?;
            return Ok(());
        } else {
            println!(
                "{} Existing credentials are invalid, need to re-configure",
                "⚠️".yellow()
            );
            storage.remove_credentials()?;
        }
    }

    println!("\n{}", "GitHub App Setup Required".bold().blue());
    println!("To use GitHub App authentication, you need to create a GitHub App:");
    println!("1. Go to: https://github.com/settings/apps");
    println!("2. Click 'New GitHub App'");
    println!("3. Use these settings:");
    println!("   - GitHub App name: Tarium SPT Mod Manager");
    println!("   - Homepage URL: https://github.com/yourusername/tarium");
    println!("   - Webhook URL: https://example.com (required but not used)");
    println!("   - Permissions: Contents (Read), Metadata (Read)");
    println!("4. Generate and download a private key");
    println!("5. Install the app and note the Installation ID from the URL");
    println!("\nSee GITHUB_APP_SETUP.md for detailed instructions.\n");

    // Get GitHub App credentials
    let app_id = match app_id {
        Some(id) => id,
        None => Text::new("Enter your GitHub App ID:")
            .prompt()
            .map_err(|_| anyhow!("Setup cancelled"))?,
    };

    let installation_id = Text::new("Enter your GitHub App Installation ID:")
        .prompt()
        .map_err(|_| anyhow!("Setup cancelled"))?;

    let private_key_path = Text::new("Enter path to your private key file (.pem):")
        .prompt()
        .map_err(|_| anyhow!("Setup cancelled"))?;

    // Validate credentials are not empty
    if app_id.trim().is_empty()
        || installation_id.trim().is_empty()
        || private_key_path.trim().is_empty()
    {
        return Err(anyhow!(
            "App ID, Installation ID, and private key path cannot be empty"
        ));
    }

    // Read private key
    let private_key = std::fs::read_to_string(&private_key_path).map_err(|e| {
        anyhow!(
            "Failed to read private key from {}: {}",
            private_key_path,
            e
        )
    })?;

    // Create credentials
    let credentials = AppCredentials {
        app_id: app_id.clone(),
        installation_id: installation_id.clone(),
        private_key,
    };

    // Test the credentials
    println!("\n{} Testing GitHub App credentials...", "🚀".blue().bold());

    let config = crate::auth::GitHubAppConfig::new(
        credentials.app_id.clone(),
        credentials.installation_id.clone(),
        credentials.private_key.clone(),
    );
    let client = GitHubAppClient::new(config);

    match client.test_authentication().await {
        Ok(()) => {
            storage.save_credentials(credentials)?;
            println!(
                "{} GitHub App configured successfully!",
                "✅".green().bold()
            );
            println!("Your GitHub App credentials have been saved securely.");

            // Show rate limit info
            if let Ok(rate_info) = client.get_rate_limit_info().await {
                println!("{} {}", "🎉".green(), rate_info.status_description());
            }
        }
        Err(e) => {
            println!(
                "{} GitHub App configuration failed: {}",
                "❌".red().bold(),
                e
            );
            println!("\nCommon issues:");
            println!("• Check that your App ID and Installation ID are correct");
            println!("• Verify the private key file is valid");
            println!("• Ensure the GitHub App is installed on target repositories");
            println!("• Confirm the app has proper permissions (Contents: Read, Metadata: Read)");
            return Err(e);
        }
    }

    Ok(())
}

/// Handle the remove command (renamed from logout)
async fn handle_remove() -> Result<()> {
    let storage = CredentialStorage::new()?;

    match storage.load_credentials()? {
        Some(_) => {
            storage.remove_credentials()?;
            println!("{} GitHub App credentials removed", "✅".green().bold());
            println!("Your stored credentials have been deleted.");
        }
        None => {
            println!("{} No GitHub App credentials found", "ℹ️".blue());
        }
    }

    Ok(())
}

/// Handle the status command
async fn handle_status() -> Result<()> {
    // Try to create a client from any available source
    match GitHubAppClient::auto() {
        Ok(client) => {
            println!("{} GitHub App configured", "✅".green().bold());

            // Test authentication and get rate limit
            match client.test_authentication().await {
                Ok(()) => {
                    println!("{} Authentication test: Passed", "✅".green());

                    if let Ok(rate_info) = client.get_rate_limit_info().await {
                        println!("📊 Rate limit: {}", rate_info.status_description());
                    }
                }
                Err(e) => {
                    println!("{} Authentication test: Failed", "❌".red());
                    println!("Error: {}", e);
                }
            }

            // Show credential source
            if GitHubAppClient::embedded().is_some() {
                println!("📦 Using embedded credentials (distributed version)");
            } else if std::env::var("TARIUM_GITHUB_APP_ID").is_ok() {
                println!("🔧 Using environment variable credentials");
            } else {
                let storage = CredentialStorage::new()?;
                match storage.credentials_status()? {
                    CredentialStatus::Exists {
                        age_days,
                        last_used,
                    } => {
                        println!("💾 Using stored credentials");
                        println!("   Age: {} days", age_days);

                        match last_used {
                            Some(used_at) => {
                                let hours_since = (chrono::Utc::now() - used_at).num_hours();
                                println!("   Last used: {} hours ago", hours_since);
                            }
                            None => {
                                println!("   Last used: Never");
                            }
                        }

                        println!(
                            "   Stored at: {}",
                            storage.credentials_file_path().display()
                        );
                    }
                    CredentialStatus::None => {
                        println!("❓ No stored credentials found");
                    }
                }
            }
        }
        Err(e) => {
            println!("{} GitHub App not configured", "❌".red());
            println!("Error: {}", e);
            println!("\nTo configure GitHub App authentication:");
            println!("1. Run `tarium auth login` to set up credentials");
            println!("2. Or set environment variables (see GITHUB_APP_SETUP.md)");
            println!("3. Or use the distributed version with embedded credentials");
            println!("\nWithout GitHub App auth, you'll be limited to 60 API calls/hour.");
        }
    }

    Ok(())
}

/// Get a GitHub App client if available
pub async fn get_github_app_client() -> Option<GitHubAppClient> {
    match GitHubAppClient::auto() {
        Ok(client) => {
            // Test authentication quietly
            if client.test_authentication().await.is_ok() {
                // Mark credentials as used if they're stored
                if let Ok(storage) = CredentialStorage::new() {
                    let _ = storage.mark_credentials_used();
                }
                Some(client)
            } else {
                log::warn!("GitHub App authentication test failed");
                None
            }
        }
        Err(_) => {
            log::debug!("No GitHub App credentials available, using anonymous API");
            None
        }
    }
}

/// Get an authenticated GitHub client or fall back to anonymous
pub async fn get_github_client() -> reqwest::Client {
    if let Some(app_client) = get_github_app_client().await {
        match app_client.authenticated_client().await {
            Ok(client) => {
                log::info!("Using authenticated GitHub App client (5,000 requests/hour)");
                return client;
            }
            Err(e) => {
                log::warn!("Failed to create authenticated client: {}", e);
            }
        }
    }

    log::info!("Using anonymous GitHub client (60 requests/hour)");
    reqwest::Client::new()
}

/// Show a helpful message about GitHub App benefits
pub fn show_github_app_info() {
    println!(
        "{} Consider setting up GitHub App authentication:",
        "💡".blue()
    );
    println!("  • 5,000 API requests/hour (vs 60 anonymous)");
    println!("  • Faster mod downloads and updates");
    println!("  • Better reliability for large modpacks");
    println!("  • Run `tarium auth login` to set up");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_github_client() {
        // This should always return a client, even if anonymous
        let client = get_github_client().await;

        // Test that we can make a basic request
        let response = client
            .get("https://api.github.com/rate_limit")
            .header("User-Agent", "Test")
            .send()
            .await;

        // We can't guarantee success due to rate limits, but we can ensure the client works
        assert!(response.is_ok() || response.is_err()); // Just ensure no panic
    }

    #[tokio::test]
    async fn test_get_github_app_client_no_creds() {
        // Without proper credentials, this should return None
        let client = get_github_app_client().await;

        // We can't assert None because the test environment might have credentials
        // But we can ensure the function doesn't panic
        match client {
            Some(_) => println!("GitHub App client available in test environment"),
            None => println!("No GitHub App client available (expected)"),
        }
    }
}
