use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use zip::read::ZipArchive;
use sevenz_rust::SevenZArchive;

/// Analyzes archive contents to extract file listings for mod tracking
pub struct ArchiveAnalyzer;

impl ArchiveAnalyzer {
    /// Extract the list of files from an archive (ZIP or 7z)
    pub fn extract_file_list(archive_path: &Path) -> Result<Vec<String>> {
        let extension = archive_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "zip" => Self::extract_zip_files(archive_path),
            "7z" => Self::extract_7z_files(archive_path),
            _ => Err(anyhow::anyhow!("Unsupported archive format: {}", extension)),
        }
    }

    /// Extract file list from a ZIP archive
    fn extract_zip_files(zip_path: &Path) -> Result<Vec<String>> {
        let file = File::open(zip_path)
            .with_context(|| format!("Failed to open ZIP file: {:?}", zip_path))?;
        let reader = BufReader::new(file);
        
        let mut archive = ZipArchive::new(reader)
            .with_context(|| format!("Failed to read ZIP archive: {:?}", zip_path))?;

        let mut files = Vec::new();
        for i in 0..archive.len() {
            let file = archive.by_index(i)
                .with_context(|| format!("Failed to access file {} in ZIP archive", i))?;
            
            if !file.name().ends_with('/') {
                // Skip directories, only add files
                files.push(file.name().to_string());
            }
        }

        Ok(files)
    }

    /// Extract file list from a 7z archive
    fn extract_7z_files(sevenz_path: &Path) -> Result<Vec<String>> {
        let file = File::open(sevenz_path)
            .with_context(|| format!("Failed to open 7z file: {:?}", sevenz_path))?;
        
        let mut archive = SevenZArchive::new(file)
            .with_context(|| format!("Failed to read 7z archive: {:?}", sevenz_path))?;

        let mut files = Vec::new();
        for entry in archive.entries() {
            let entry = entry.with_context(|| "Failed to read 7z archive entry")?;
            
            if !entry.is_dir() {
                // Skip directories, only add files
                if let Some(path) = entry.path().file_name() {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        }

        Ok(files)
    }

    /// Extract files from an archive to a target directory and track them
    pub fn extract_and_track_files(
        archive_path: &Path,
        target_dir: &Path,
        mod_name: &str,
    ) -> Result<Vec<String>> {
        let files = Self::extract_file_list(archive_path)?;
        let mut extracted_files = Vec::new();

        // Create target directory if it doesn't exist
        std::fs::create_dir_all(target_dir)
            .with_context(|| format!("Failed to create target directory: {:?}", target_dir))?;

        let extension = archive_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "zip" => Self::extract_zip_to_dir(archive_path, target_dir, &files, &mut extracted_files)?,
            "7z" => Self::extract_7z_to_dir(archive_path, target_dir, &files, &mut extracted_files)?,
            _ => return Err(anyhow::anyhow!("Unsupported archive format: {}", extension)),
        }

        Ok(extracted_files)
    }

    /// Extract ZIP archive to directory and track extracted files
    fn extract_zip_to_dir(
        zip_path: &Path,
        target_dir: &Path,
        file_list: &[String],
        extracted_files: &mut Vec<String>,
    ) -> Result<()> {
        let file = File::open(zip_path)
            .with_context(|| format!("Failed to open ZIP file: {:?}", zip_path))?;
        let reader = BufReader::new(file);
        
        let mut archive = ZipArchive::new(reader)
            .with_context(|| format!("Failed to read ZIP archive: {:?}", zip_path))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .with_context(|| format!("Failed to access file {} in ZIP archive", i))?;
            
            if !file.name().ends_with('/') {
                // Skip directories, only extract files
                let output_path = target_dir.join(file.name());
                
                // Ensure parent directory exists
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent directory: {:?}", parent))?;
                }

                let mut output_file = std::fs::File::create(&output_path)
                    .with_context(|| format!("Failed to create output file: {:?}", output_path))?;
                
                std::io::copy(&mut file, &mut output_file)
                    .with_context(|| format!("Failed to extract file: {:?}", file.name()))?;

                // Track the extracted file relative to the target directory
                if let Ok(relative_path) = output_path.strip_prefix(target_dir) {
                    extracted_files.push(relative_path.to_string_lossy().to_string());
                }
            }
        }

        Ok(())
    }

    /// Extract 7z archive to directory and track extracted files
    fn extract_7z_to_dir(
        sevenz_path: &Path,
        target_dir: &Path,
        file_list: &[String],
        extracted_files: &mut Vec<String>,
    ) -> Result<()> {
        let file = File::open(sevenz_path)
            .with_context(|| format!("Failed to open 7z file: {:?}", sevenz_path))?;
        
        let mut archive = SevenZArchive::new(file)
            .with_context(|| format!("Failed to read 7z archive: {:?}", sevenz_path))?;

        for entry in archive.entries() {
            let mut entry = entry.with_context(|| "Failed to read 7z archive entry")?;
            
            if !entry.is_dir() {
                let entry_path = entry.path();
                let output_path = target_dir.join(entry_path);
                
                // Ensure parent directory exists
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent directory: {:?}", parent))?;
                }

                let mut output_file = std::fs::File::create(&output_path)
                    .with_context(|| format!("Failed to create output file: {:?}", output_path))?;
                
                std::io::copy(&mut entry, &mut output_file)
                    .with_context(|| format!("Failed to extract file: {:?}", entry_path))?;

                // Track the extracted file relative to the target directory
                if let Ok(relative_path) = output_path.strip_prefix(target_dir) {
                    extracted_files.push(relative_path.to_string_lossy().to_string());
                }
            }
        }

        Ok(())
    }

    /// Filter files to include only relevant mod files (excluding metadata, etc.)
    pub fn filter_mod_files(files: Vec<String>) -> Vec<String> {
        files.into_iter()
            .filter(|file| {
                let file_lower = file.to_lowercase();
                
                // Include common mod file types
                file_lower.ends_with(".dll") ||
                file_lower.ends_with(".json") ||
                file_lower.ends_with(".bundle") ||
                file_lower.ends_with(".cfg") ||
                file_lower.ends_with(".config") ||
                
                // Include common plugin file extensions
                file_lower.ends_with(".plugin") ||
                
                // Include common asset file types
                file_lower.ends_with(".png") ||
                file_lower.ends_with(".jpg") ||
                file_lower.ends_with(".jpeg") ||
                
                // Include common SPT-specific file patterns
                file_lower.contains("plugin") ||
                file_lower.contains("mod") ||
                file_lower.contains("spt") ||
                
                // Exclude common non-mod files
                !file_lower.contains("readme") &&
                !file_lower.contains("license") &&
                !file_lower.contains("changelog") &&
                !file_lower.contains("__macosx") &&
                !file_lower.contains(".ds_store") &&
                !file_lower.starts_with('.') && // Hidden files
                !file_lower.ends_with(".md") &&
                !file_lower.ends_with(".txt") &&
                !file_lower.ends_with(".pdf")
            })
            .collect()
    }

    /// Analyze an archive and return filtered mod files
    pub fn analyze_mod_archive(archive_path: &Path) -> Result<Vec<String>> {
        let all_files = Self::extract_file_list(archive_path)?;
        let mod_files = Self::filter_mod_files(all_files);
        Ok(mod_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    #[test]
    fn test_filter_mod_files() {
        let files = vec![
            "mod.dll".to_string(),
            "config.json".to_string(),
            "plugin.bundle".to_string(),
            "README.md".to_string(),
            "LICENSE.txt".to_string(),
            ".DS_Store".to_string(),
            "spt_plugin.dll".to_string(),
            "assets/icon.png".to_string(),
            "docs/changelog.pdf".to_string(),
        ];

        let filtered = ArchiveAnalyzer::filter_mod_files(files);
        
        assert!(filtered.contains(&"mod.dll".to_string()));
        assert!(filtered.contains(&"config.json".to_string()));
        assert!(filtered.contains(&"plugin.bundle".to_string()));
        assert!(filtered.contains(&"spt_plugin.dll".to_string()));
        assert!(filtered.contains(&"assets/icon.png".to_string()));
        
        assert!(!filtered.contains(&"README.md".to_string()));
        assert!(!filtered.contains(&"LICENSE.txt".to_string()));
        assert!(!filtered.contains(&".DS_Store".to_string()));
        assert!(!filtered.contains(&"docs/changelog.pdf".to_string()));
    }

    #[test]
    fn test_extract_zip_files() -> Result<()> {
        // Create a temporary ZIP file for testing
        let mut temp_file = NamedTempFile::new()?;
        let mut zip = ZipWriter::new(&mut temp_file);
        
        zip.add_directory("test_dir/", FileOptions::default())?;
        zip.start_file("test_file.txt", FileOptions::default())?;
        zip.write_all(b"Hello, World!")?;
        zip.start_file("test_dir/nested_file.txt", FileOptions::default())?;
        zip.write_all(b"Nested content")?;
        zip.finish()?;
        
        temp_file.as_file().sync_all()?;
        
        let files = ArchiveAnalyzer::extract_file_list(temp_file.path())?;
        
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"test_file.txt".to_string()));
        assert!(files.contains(&"test_dir/nested_file.txt".to_string()));
        
        Ok(())
    }
}