use std::fs;
use std::path::Path;
use tempfile::TempDir;

use libarov::config::mod_state::ModStateManager;
use libarov::config::structs::{Mod, ModIdentifier, Profile};

#[test]
fn test_mod_state_manager() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    let manager = ModStateManager::new(output_dir.clone());

    // Test directory creation
    manager.ensure_disabled_dir().unwrap();
    assert!(manager.disabled_dir().exists());

    // Test mod disabled directory creation
    let mod_dir = manager.mod_disabled_dir("test_mod");
    assert!(mod_dir.ends_with("disabled-mods/test_mod"));
}

#[test]
fn test_enable_disable_mod() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    let manager = ModStateManager::new(output_dir.clone());

    // Create test files
    let test_files = vec!["test_file1.txt", "test_file2.txt"];
    for file in &test_files {
        let file_path = output_dir.join(file);
        fs::write(file_path, b"test content").unwrap();
    }

    // Disable mod
    manager.disable_mod("test_mod", &test_files).unwrap();

    // Check files are moved to disabled directory
    let mod_disabled_dir = manager.mod_disabled_dir("test_mod");
    assert!(mod_disabled_dir.exists());

    for file in &test_files {
        let original_path = output_dir.join(file);
        let disabled_path = mod_disabled_dir.join(file);
        assert!(!original_path.exists());
        assert!(disabled_path.exists());
    }

    // Enable mod back
    manager.enable_mod("test_mod", &test_files).unwrap();

    // Check files are moved back
    for file in &test_files {
        let original_path = output_dir.join(file);
        let disabled_path = mod_disabled_dir.join(file);
        assert!(original_path.exists());
        assert!(!disabled_path.exists());
    }
}

#[test]
fn test_list_disabled_mods() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    let manager = ModStateManager::new(output_dir.clone());

    // Initially no disabled mods
    let disabled_mods = manager.list_disabled_mods().unwrap();
    assert!(disabled_mods.is_empty());

    // Create and disable a mod
    fs::write(output_dir.join("test.txt"), b"test").unwrap();
    manager.disable_mod("test_mod", &["test.txt".to_string()]).unwrap();

    // Check disabled mod is listed
    let disabled_mods = manager.list_disabled_mods().unwrap();
    assert_eq!(disabled_mods.len(), 1);
    assert_eq!(disabled_mods[0], "test_mod");
}

#[test]
fn test_is_mod_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    let manager = ModStateManager::new(output_dir.clone());

    let test_files = vec!["enabled_file.txt"];

    // Initially no files, so mod should be considered disabled
    assert!(!manager.is_mod_enabled(&test_files));

    // Create file in enabled directory
    fs::write(output_dir.join("enabled_file.txt"), b"test").unwrap();

    // Now mod should be considered enabled
    assert!(manager.is_mod_enabled(&test_files));

    // Disable the mod
    manager.disable_mod("test_mod", &test_files).unwrap();

    // Now mod should be considered disabled
    assert!(!manager.is_mod_enabled(&test_files));
}

#[test]
fn test_cleanup_mod_disabled_dir() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();
    let manager = ModStateManager::new(output_dir.clone());

    // Create and disable a mod
    fs::write(output_dir.join("test.txt"), b"test").unwrap();
    manager.disable_mod("test_mod", &["test.txt".to_string()]).unwrap();

    // Check disabled directory exists
    let mod_disabled_dir = manager.mod_disabled_dir("test_mod");
    assert!(mod_disabled_dir.exists());

    // Cleanup
    manager.cleanup_mod_disabled_dir("test_mod").unwrap();

    // Check directory is removed
    assert!(!mod_disabled_dir.exists());
}

#[test]
fn test_mod_struct_with_enabled_field() {
    let mod_ = Mod::new(
        "Test Mod".to_string(),
        ModIdentifier::GitHubRepository("test".to_string(), "mod".to_string()),
        vec![],
    );

    // Default enabled state should be true
    assert!(mod_.enabled);

    // Files should be empty by default
    assert!(mod_.files.is_empty());
}

#[test]
fn test_profile_with_mods() {
    let mut profile = Profile::new(
        "Test Profile".to_string(),
        PathBuf::from("/tmp/test"),
        vec!["3.10".to_string()],
        false,
    );

    // Add a mod
    profile.push_mod(
        "Test Mod".to_string(),
        ModIdentifier::GitHubRepository("test".to_string(), "mod".to_string()),
        "test_mod".to_string(),
    );

    // Check mod was added with default enabled state
    assert_eq!(profile.mods.len(), 1);
    assert!(profile.mods[0].enabled);
    assert!(profile.mods[0].files.is_empty());
}