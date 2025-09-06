//! Storage for GitHub App credentials
//!
//! This module handles secure storage and retrieval of GitHub App credentials
//! in the user's configuration directory.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// GitHub App credentials storage
#[derive(Debug)]
pub struct CredentialStorage {
    config_dir: PathBuf,
    credentials_file: PathBuf,
}

/// GitHub App credentials
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppCredentials {
    pub app_id: String,
    pub installation_id: String,
    pub private_key: String,
}

/// Stored credentials with metadata
#[derive(Debug, Deserialize, Serialize)]
struct StoredCredentials {
    credentials: AppCredentials,
    created_at: chrono::DateTime<chrono::Utc>,
    last_used: Option<chrono::DateTime<chrono::Utc>>,
}

impl CredentialStorage {
    /// Create new credential storage instance
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let credentials_file = config_dir.join("github_app.json");

        // Ensure config directory exists
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(Self {
            config_dir,
            credentials_file,
        })
    }

    /// Get the configuration directory for Tarium
    fn get_config_dir() -> Result<PathBuf> {
        #[cfg(debug_assertions)]
        let app_name = "tarium-dev";
        #[cfg(not(debug_assertions))]
        let app_name = "tarium";

        if let Some(config_dir) = dirs::config_dir() {
            Ok(config_dir.join(app_name))
        } else {
            // Fallback for systems without standard config directory
            let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
            Ok(home.join(format!(".{}", app_name)))
        }
    }

    /// Save credentials to storage
    pub fn save_credentials(&self, credentials: AppCredentials) -> Result<()> {
        let stored_credentials = StoredCredentials {
            credentials,
            created_at: chrono::Utc::now(),
            last_used: None,
        };

        let json = serde_json::to_string_pretty(&stored_credentials)?;
        fs::write(&self.credentials_file, json)?;

        log::info!(
            "GitHub App credentials saved to {:?}",
            self.credentials_file
        );
        Ok(())
    }

    /// Load credentials from storage
    pub fn load_credentials(&self) -> Result<Option<AppCredentials>> {
        if !self.credentials_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.credentials_file)?;
        let stored_credentials: StoredCredentials = serde_json::from_str(&content)?;

        // Check if credentials are too old (older than 1 year)
        let age = chrono::Utc::now() - stored_credentials.created_at;
        if age > chrono::Duration::days(365) {
            log::warn!("GitHub App credentials are over 1 year old, may need refresh");
        }

        Ok(Some(stored_credentials.credentials))
    }

    /// Update the last used time for the stored credentials
    pub fn mark_credentials_used(&self) -> Result<()> {
        if !self.credentials_file.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.credentials_file)?;
        let mut stored_credentials: StoredCredentials = serde_json::from_str(&content)?;

        stored_credentials.last_used = Some(chrono::Utc::now());

        let json = serde_json::to_string_pretty(&stored_credentials)?;
        fs::write(&self.credentials_file, json)?;

        Ok(())
    }

    /// Remove stored credentials
    pub fn remove_credentials(&self) -> Result<()> {
        if self.credentials_file.exists() {
            fs::remove_file(&self.credentials_file)?;
            log::info!("GitHub App credentials removed from storage");
        }
        Ok(())
    }

    /// Check if credentials exist and when they were last used
    pub fn credentials_status(&self) -> Result<CredentialStatus> {
        if !self.credentials_file.exists() {
            return Ok(CredentialStatus::None);
        }

        let content = fs::read_to_string(&self.credentials_file)?;
        let stored_credentials: StoredCredentials = serde_json::from_str(&content)?;

        let age = chrono::Utc::now() - stored_credentials.created_at;
        let last_used = stored_credentials.last_used;

        Ok(CredentialStatus::Exists {
            age_days: age.num_days(),
            last_used,
        })
    }

    /// Get the path to the credentials file (for debugging)
    pub fn credentials_file_path(&self) -> &PathBuf {
        &self.credentials_file
    }

    /// Get the config directory path
    pub fn config_dir_path(&self) -> &PathBuf {
        &self.config_dir
    }
}

/// Status of the stored credentials
#[derive(Debug)]
pub enum CredentialStatus {
    None,
    Exists {
        age_days: i64,
        last_used: Option<chrono::DateTime<chrono::Utc>>,
    },
}

impl Default for CredentialStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create credential storage")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (CredentialStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = CredentialStorage {
            config_dir: temp_dir.path().to_path_buf(),
            credentials_file: temp_dir.path().join("github_app.json"),
        };
        (storage, temp_dir)
    }

    #[test]
    fn test_save_and_load_credentials() {
        let (storage, _temp_dir) = create_test_storage();

        let credentials = AppCredentials {
            app_id: "12345".to_string(),
            installation_id: "67890".to_string(),
            private_key: "test-private-key".to_string(),
        };

        // Save credentials
        storage.save_credentials(credentials.clone()).unwrap();

        // Load credentials
        let loaded_credentials = storage.load_credentials().unwrap();
        assert!(loaded_credentials.is_some());

        let loaded_credentials = loaded_credentials.unwrap();
        assert_eq!(loaded_credentials.app_id, credentials.app_id);
        assert_eq!(
            loaded_credentials.installation_id,
            credentials.installation_id
        );
        assert_eq!(loaded_credentials.private_key, credentials.private_key);
    }

    #[test]
    fn test_credentials_status_none() {
        let (storage, _temp_dir) = create_test_storage();

        let status = storage.credentials_status().unwrap();
        matches!(status, CredentialStatus::None);
    }

    #[test]
    fn test_remove_credentials() {
        let (storage, _temp_dir) = create_test_storage();

        let credentials = AppCredentials {
            app_id: "12345".to_string(),
            installation_id: "67890".to_string(),
            private_key: "test-private-key".to_string(),
        };

        // Save and then remove credentials
        storage.save_credentials(credentials).unwrap();
        assert!(storage.credentials_file.exists());

        storage.remove_credentials().unwrap();
        assert!(!storage.credentials_file.exists());
    }
}
