use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Manages the disabled mods directory structure and file operations
pub struct ModStateManager {
    profile_output_dir: PathBuf,
}

impl ModStateManager {
    /// Create a new ModStateManager for the given profile output directory
    pub fn new(profile_output_dir: PathBuf) -> Self {
        Self {
            profile_output_dir,
        }
    }

    /// Get the path to the disabled mods directory
    pub fn disabled_dir(&self) -> PathBuf {
        self.profile_output_dir.join("disabled-mods")
    }

    /// Get the path to the enabled mods directory (main mods directory)
    pub fn enabled_dir(&self) -> &Path {
        &self.profile_output_dir
    }

    /// Ensure the disabled mods directory exists
    pub fn ensure_disabled_dir(&self) -> Result<()> {
        let disabled_dir = self.disabled_dir();
        if !disabled_dir.exists() {
            fs::create_dir_all(&disabled_dir)
                .with_context(|| format!("Failed to create disabled mods directory: {:?}", disabled_dir))?;
        }
        Ok(())
    }

    /// Get the path for storing disabled files for a specific mod
    pub fn mod_disabled_dir(&self, mod_name: &str) -> PathBuf {
        self.disabled_dir().join(self::sanitize_filename(mod_name))
    }

    /// Move files from enabled to disabled state for a mod
    pub fn disable_mod(&self, mod_name: &str, files: &[String]) -> Result<()> {
        self.ensure_disabled_dir()?;
        
        let mod_disabled_dir = self.mod_disabled_dir(mod_name);
        fs::create_dir_all(&mod_disabled_dir)
            .with_context(|| format!("Failed to create mod disabled directory: {:?}", mod_disabled_dir))?;

        for file in files {
            let src_path = self.enabled_dir().join(file);
            let dst_path = mod_disabled_dir.join(file);

            if src_path.exists() {
                // Ensure parent directory exists
                if let Some(parent) = dst_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent directory: {:?}", parent))?;
                }

                fs::rename(&src_path, &dst_path)
                    .with_context(|| format!("Failed to move file from {:?} to {:?}", src_path, dst_path))?;
            }
        }

        Ok(())
    }

    /// Move files from disabled to enabled state for a mod
    pub fn enable_mod(&self, mod_name: &str, files: &[String]) -> Result<()> {
        let mod_disabled_dir = self.mod_disabled_dir(mod_name);

        for file in files {
            let src_path = mod_disabled_dir.join(file);
            let dst_path = self.enabled_dir().join(file);

            if src_path.exists() {
                // Ensure parent directory exists
                if let Some(parent) = dst_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent directory: {:?}", parent))?;
                }

                fs::rename(&src_path, &dst_path)
                    .with_context(|| format!("Failed to move file from {:?} to {:?}", src_path, dst_path))?;
            }
        }

        // Clean up empty mod disabled directory
        if mod_disabled_dir.exists() {
            if let Ok(mut entries) = fs::read_dir(&mod_disabled_dir) {
                if entries.next().is_none() {
                    fs::remove_dir(&mod_disabled_dir)
                        .with_context(|| format!("Failed to remove empty mod disabled directory: {:?}", mod_disabled_dir))?;
                }
            }
        }

        Ok(())
    }

    /// Check if a mod is currently enabled by checking if its files exist in the enabled directory
    pub fn is_mod_enabled(&self, files: &[String]) -> bool {
        files.iter().any(|file| {
            let enabled_path = self.enabled_dir().join(file);
            enabled_path.exists()
        })
    }

    /// Get the list of disabled mod directories
    pub fn list_disabled_mods(&self) -> Result<Vec<String>> {
        let disabled_dir = self.disabled_dir();
        if !disabled_dir.exists() {
            return Ok(Vec::new());
        }

        let mut disabled_mods = Vec::new();
        for entry in fs::read_dir(&disabled_dir)
            .with_context(|| format!("Failed to read disabled mods directory: {:?}", disabled_dir))?
        {
            let entry = entry.with_context(|| "Failed to read directory entry")?;
            if entry.file_type().map_or(false, |ft| ft.is_dir()) {
                if let Some(name) = entry.file_name().to_str() {
                    disabled_mods.push(name.to_string());
                }
            }
        }

        Ok(disabled_mods)
    }

    /// Remove a mod's disabled directory (useful when removing a mod entirely)
    pub fn cleanup_mod_disabled_dir(&self, mod_name: &str) -> Result<()> {
        let mod_disabled_dir = self.mod_disabled_dir(mod_name);
        if mod_disabled_dir.exists() {
            fs::remove_dir_all(&mod_disabled_dir)
                .with_context(|| format!("Failed to remove mod disabled directory: {:?}", mod_disabled_dir))?;
        }
        Ok(())
    }
}

/// Sanitize a filename to be safe for filesystem operations
fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test_mod"), "test_mod");
        assert_eq!(sanitize_filename("test/mod"), "test_mod");
        assert_eq!(sanitize_filename("test*mod?"), "test_mod_");
        assert_eq!(sanitize_filename("Test: Mod"), "Test_ Mod");
    }

    #[test]
    fn test_mod_state_manager() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_dir = temp_dir.path().to_path_buf();
        let manager = ModStateManager::new(output_dir.clone());

        // Test directory creation
        manager.ensure_disabled_dir()?;
        assert!(manager.disabled_dir().exists());

        // Test mod disabled directory
        let mod_dir = manager.mod_disabled_dir("test_mod");
        assert!(mod_dir.ends_with("disabled-mods/test_mod"));

        Ok(())
    }
}