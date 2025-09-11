use crate::{
    default_semaphore,
    download::{clean, download},
    CROSS, SEMAPHORE, STYLE_NO, TICK,
};
use anyhow::{anyhow, bail, Result};
use colored::Colorize as _;
// use indicatif::ProgressBar; // Temporarily disabled progress bar
use libarov::{
    archive_analyzer::ArchiveAnalyzer,
    config::{
        filters::ProfileParameters as _,
        mod_state::ModStateManager,
        structs::{Mod, ModIdentifier, Profile},
    },
    upgrade::{mod_downloadable, DownloadData},
};
use log::{debug, info, warn};
use parking_lot::Mutex;
use sevenz_rust::decompress_file;
use std::collections::HashSet;
use std::{fs::read_dir, mem::take, sync::Arc, time::Duration};
use std::{
    fs::{self, copy as fs_copy, create_dir_all, File},
    path::{Path, PathBuf},
};
use tokio::task::JoinSet;
use zip::ZipArchive;

#[cfg(windows)]
fn normalize_permissions(path: &Path) {
    if let Ok(meta) = std::fs::metadata(path) {
        if meta.permissions().readonly() {
            let mut perms = meta.permissions();
            perms.set_readonly(false);
            let _ = std::fs::set_permissions(path, perms);
        }
    }
}
#[cfg(unix)]
fn normalize_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        // Directories need execute bit; detect dir
        if path.is_dir() {
            let _ = perms.set_mode(0o755);
        } else {
            let _ = perms.set_mode(0o644);
        }
        let _ = std::fs::set_permissions(path, perms);
    }
}

/// Normalize the permissions of all files and directories in a directory tree.
fn normalize_tree(root: &Path) {
    if !root.exists() {
        return;
    }
    if root.is_dir() {
        normalize_permissions(root);
        if let Ok(rd) = std::fs::read_dir(root) {
            for e in rd.flatten() {
                normalize_tree(&e.path());
            }
        }
    } else {
        normalize_permissions(root);
    }
}

/// Move a processed archive to the archive store.
fn move_processed_archive(from: &Path, archive_store: &Path) -> Result<()> {
    let target = archive_store.join(from.file_name().unwrap_or_default());
    if target.exists() {
        let _ = fs::remove_file(&target); // best-effort remove existing
    }

    match fs::rename(from, &target) {
        Ok(_) => {
            info!(SCOPE = "subcommands::upgrade", from:display = from.display().to_string(), to:display = target.display().to_string(); "moved archive to store");
            Ok(())
        }
        Err(_) => {
            // Fallback: copy then delete original

            info!(SCOPE = "subcommands::upgrade", from:display = from.display().to_string(), to:display = target.display().to_string(); "rename failed; copying then removing");
            fs_copy(from, &target)?;

            fs::remove_file(from)?;

            normalize_permissions(&target);

            Ok(())
        }
    }
}

/// Update mod files list after upgrade/extract process
fn update_mod_files(profile: &mut Profile) -> Result<()> {
    let mod_state_manager = ModStateManager::new(profile.output_dir.clone());
    
    for mod_ in &mut profile.mods {
        if mod_.enabled {
            // For enabled mods, scan the output directory for files that might belong to this mod
            let mut mod_files = Vec::new();
            
            // Common patterns for SPT mods
            let search_patterns = &[
                format!("BepInEx/plugins/{}.dll", mod_.name.to_lowercase()),
                format!("user/mods/{}", mod_.name.to_lowercase()),
                format!("{}.dll", mod_.name.to_lowercase()),
            ];
            
            // Also check for any DLLs or plugins that might belong to this mod
            if let Ok(entries) = std::fs::read_dir(&profile.output_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            let file_lower = file_name.to_lowercase();
                            
                            // Check if file matches mod name or common patterns
                            if file_lower.contains(&mod_.name.to_lowercase()) ||
                               file_lower.ends_with(".dll") ||
                               file_lower.ends_with(".plugin") {
                               
                                // Get relative path from output directory
                                if let Ok(relative_path) = path.strip_prefix(&profile.output_dir) {
                                    mod_files.push(relative_path.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
            
            // Also check BepInEx/plugins and user/mods directories
            for subdir in ["BepInEx/plugins", "user/mods"] {
                let subdir_path = profile.output_dir.join(subdir);
                if subdir_path.exists() {
                    if let Ok(entries) = std::fs::read_dir(&subdir_path) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                                    let file_lower = file_name.to_lowercase();
                                    
                                    if file_lower.contains(&mod_.name.to_lowercase()) {
                                        if let Ok(relative_path) = path.strip_prefix(&profile.output_dir) {
                                            mod_files.push(relative_path.to_string_lossy().to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Update mod files list
            mod_.files = mod_files;
        } else {
            // For disabled mods, files should already be tracked and in disabled directory
            // No need to update as they're not active
        }
    }
    
    Ok(())
}

fn extract_all_archives(output_dir: &Path) -> Result<()> {
    ensure_required_dirs(output_dir)?;
    let archive_store = output_dir.join("MODS");
    if !archive_store.exists() {
        if let Err(e) = create_dir_all(&archive_store) {
            println!("{} Failed creating MODS dir: {}", CROSS.red(), e);
        }
    }

    let mut installation_errors = Vec::new();
    let mut move_errors = Vec::new();

    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext.to_ascii_lowercase().as_str() {
                    "zip" => match extract_zip(&path, output_dir) {
                        Ok(_) => {
                            if let Err(e) = move_processed_archive(&path, &archive_store) {
                                move_errors.push((path.clone(), e));
                                println!(
                                    "{} Extracted (move failed: {}) {}",
                                    CROSS.red(),
                                    move_errors.last().unwrap().1,
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                );
                            } else {
                                println!(
                                    "{} Extracted (moved) {}",
                                    TICK.clone(),
                                    path.file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .dimmed()
                                );
                            }
                        }
                        Err(e) => {
                            installation_errors.push((path.clone(), e));
                            println!(
                                "{} Failed extracting {}: {}",
                                CROSS.red(),
                                path.file_name().unwrap_or_default().to_string_lossy(),
                                installation_errors.last().unwrap().1
                            );
                        }
                    },
                    "7z" => match extract_7z(&path, output_dir) {
                        Ok(_) => {
                            if let Err(e) = move_processed_archive(&path, &archive_store) {
                                move_errors.push((path.clone(), e));
                                println!(
                                    "{} Extracted (move failed: {}) {}",
                                    CROSS.red(),
                                    move_errors.last().unwrap().1,
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                );
                            } else {
                                println!(
                                    "{} Extracted (moved) {}",
                                    TICK.clone(),
                                    path.file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .dimmed()
                                );
                            }
                        }
                        Err(e) => {
                            installation_errors.push((path.clone(), e));
                            println!(
                                "{} Failed extracting {}: {}",
                                CROSS.red(),
                                path.file_name().unwrap_or_default().to_string_lossy(),
                                installation_errors.last().unwrap().1
                            );
                        }
                    },
                    _ => {
                        // Not an archive we handle
                        debug!(SCOPE = "subcommands::upgrade", path:display = path.display().to_string(); "skipping non-archive file for now");
                    }
                }
            }
        }
    }

    // Report installation errors
    if !installation_errors.is_empty() {
        for (path, err) in &installation_errors {
            // Check if this is a "no components" error vs other errors
            if err
                .to_string()
                .contains("no mod components found to install")
            {
                debug!(SCOPE = "subcommands::upgrade", path:display = path.display().to_string(), error:display = err.to_string(); "mod archive had no installable components - likely wrapper folder issue");
            } else {
                println!(
                    "{} Failed to install {}: {}",
                    CROSS.red(),
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    err
                );
            }
        }

        // Only fail for non-wrapper related errors
        let critical_errors: Vec<_> = installation_errors
            .iter()
            .filter(|(_, err)| {
                !err.to_string()
                    .contains("no mod components found to install")
            })
            .collect();

        if !critical_errors.is_empty() {
            let error_details = critical_errors
                .iter()
                .map(|(path, err)| {
                    format!(
                        "  - {}: {}",
                        path.file_name().unwrap_or_default().to_string_lossy(),
                        err
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");

            return Err(anyhow!(
                "Failed to install {} mod archive(s) due to critical errors:\n{}",
                critical_errors.len(),
                error_details
            ));
        }
    }

    // Move errors are warnings but don't stop the process
    if !move_errors.is_empty() {
        println!(
            "{} {} archives were extracted successfully but could not be moved to MODS directory",
            "Warning:".yellow(),
            move_errors.len()
        );
    }

    Ok(())
}

fn extract_7z(archive_path: &Path, output_dir: &Path) -> Result<()> {
    let temp_dir = output_dir
        .join(".extract_tmp")
        .join(archive_path.file_stem().unwrap_or_default());

    if temp_dir.exists() {
        info!(SCOPE = "subcommands::upgrade", path:display = temp_dir.display().to_string(); "removing pre-existing temp dir");
        fs::remove_dir_all(&temp_dir)?;
    }

    info!(SCOPE = "subcommands::upgrade", path:display = temp_dir.display().to_string(); "creating temp dir for 7z");

    create_dir_all(&temp_dir)?;

    info!(SCOPE = "subcommands::upgrade", from:display = archive_path.display().to_string(), to:display = temp_dir.display().to_string(); "decompressing 7z archive");

    decompress_file(archive_path, &temp_dir)?;

    // quick permission normalization on extracted tree

    fn walk_and_normalize(p: &Path) {
        if let Ok(rd) = std::fs::read_dir(p) {
            for e in rd.flatten() {
                let path = e.path();

                if path.is_dir() {
                    walk_and_normalize(&path);
                } else {
                    normalize_permissions(&path);
                }
            }
        }
    }

    walk_and_normalize(&temp_dir);

    info!(SCOPE = "subcommands::upgrade", path:display = temp_dir.display().to_string(); "installing extracted contents");

    install_extracted(&temp_dir, output_dir)?;

    info!(SCOPE = "subcommands::upgrade", path:display = temp_dir.display().to_string(); "cleaning temp dir after 7z install");

    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

fn extract_zip(zip_path: &Path, output_dir: &Path) -> Result<()> {
    debug!(SCOPE = "subcommands::upgrade", path:display = zip_path.display().to_string(); "opening zip for extraction");

    let file = File::open(zip_path)?;

    let mut archive = ZipArchive::new(file)?;

    let temp_dir = output_dir
        .join(".extract_tmp")
        .join(zip_path.file_stem().unwrap_or_default());

    if temp_dir.exists() {
        info!(SCOPE = "subcommands::upgrade", path:display = temp_dir.display().to_string(); "removing pre-existing temp dir");
        fs::remove_dir_all(&temp_dir)?;
    }

    create_dir_all(&temp_dir)?;

    info!(SCOPE = "subcommands::upgrade", path:display = temp_dir.display().to_string(); "created temp dir for zip");

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let outpath = temp_dir.join(file.mangled_name());

        if file.is_dir() {
            debug!(SCOPE = "subcommands::upgrade", path:display = outpath.display().to_string(); "creating directory from zip entry");

            create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                create_dir_all(parent)?;
            }

            debug!(SCOPE = "subcommands::upgrade", path:display = outpath.display().to_string(); "creating file from zip entry");

            let mut outfile = File::create(&outpath)?;

            std::io::copy(&mut file, &mut outfile)?;

            normalize_permissions(&outpath);
        }
    }

    info!(SCOPE = "subcommands::upgrade", path:display = temp_dir.display().to_string(); "installing extracted contents");

    install_extracted(&temp_dir, output_dir)?;

    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

fn install_extracted(temp_dir: &Path, output_dir: &Path) -> Result<()> {
    debug!(SCOPE = "subcommands::upgrade", temp_dir:display = temp_dir.display().to_string(), output_dir:display = output_dir.display().to_string(); "starting mod installation from extracted contents");

    let mut installation_count = 0;

    // Collapse single-folder wrappers with same name as archive
    let mut root = temp_dir.to_path_buf();
    let original_root = root.clone();

    // Get the archive name without extension for comparison
    let archive_name = temp_dir.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Only collapse if there's a single directory with the same name as the archive
    match fs::read_dir(&root) {
        Ok(entries) => {
            let collected: Vec<_> = entries.filter_map(|e| e.ok()).collect();

            // Check for single directory with same name as archive
            if collected.len() == 1 {
                let entry = &collected[0];
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Some(dir_name) = entry.file_name().to_str() {
                        // Only collapse if directory name matches archive name (wrapper folder pattern)
                        if dir_name == archive_name {
                            root = entry.path();
                            debug!(SCOPE = "subcommands::upgrade", original_root:display = original_root.display().to_string(), new_root:display = root.display().to_string(); "collapsed same-name wrapper folder");
                        }
                    }
                }
            }
        }
        Err(e) => {
            debug!(SCOPE = "subcommands::upgrade", temp_dir:display = temp_dir.display().to_string(), error:display = e.to_string(); "failed to read temp directory for wrapper collapse");
        }
    }

    // Merge BepInEx if present
    let bep = root.join("BepInEx");
    if bep.exists() {
        match copy_dir_recursive(&bep, &output_dir.join("BepInEx")) {
            Ok(_) => {
                info!(SCOPE = "subcommands::upgrade", from:display = bep.display().to_string(), to:display = output_dir.join("BepInEx").display().to_string(); "installed BepInEx directory");
                installation_count += 1;
            }
            Err(e) => {
                debug!(SCOPE = "subcommands::upgrade", from:display = bep.display().to_string(), to:display = output_dir.join("BepInEx").display().to_string(), error:display = e.to_string(); "failed to copy BepInEx directory");
                return Err(e);
            }
        }
    } else {
        debug!(SCOPE = "subcommands::upgrade", root:display = root.display().to_string(); "no BepInEx directory found in extracted mod");
    }

    // Merge user mods
    let user_dir = root.join("user");
    if user_dir.exists() {
        match copy_dir_recursive(&user_dir, &output_dir.join("user")) {
            Ok(_) => {
                info!(SCOPE = "subcommands::upgrade", from:display = user_dir.display().to_string(), to:display = output_dir.join("user").display().to_string(); "installed user directory");
                installation_count += 1;
            }
            Err(e) => {
                debug!(SCOPE = "subcommands::upgrade", from:display = user_dir.display().to_string(), to:display = output_dir.join("user").display().to_string(), error:display = e.to_string(); "failed to copy user directory");
                return Err(e);
            }
        }
    } else {
        debug!(SCOPE = "subcommands::upgrade", root:display = root.display().to_string(); "no user directory found in extracted mod");
    }

    // Top-level dlls => BepInEx/plugins
    let plugins_dir = output_dir.join("BepInEx").join("plugins");
    if let Err(e) = create_dir_all(&plugins_dir) {
        warn!(SCOPE = "subcommands::upgrade", plugins_dir:display = plugins_dir.display().to_string(), error:display = e.to_string(); "failed to create plugins directory");
        return Err(e.into());
    }

    let mut dll_count = 0;
    match fs::read_dir(&root) {
        Ok(entries) => {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file()
                    && p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("dll"))
                        .unwrap_or(false)
                {
                    let target = plugins_dir.join(p.file_name().unwrap());
                    match fs_copy(&p, &target) {
                        Ok(_) => {
                            normalize_permissions(&target);
                            debug!(SCOPE = "subcommands::upgrade", from:display = p.display().to_string(), to:display = target.display().to_string(); "installed DLL plugin");
                            dll_count += 1;
                        }
                        Err(e) => {
                            warn!(SCOPE = "subcommands::upgrade", from:display = p.display().to_string(), to:display = target.display().to_string(), error:display = e.to_string(); "failed to copy DLL plugin");
                            return Err(e.into());
                        }
                    }
                }
            }
        }
        Err(e) => {
            warn!(SCOPE = "subcommands::upgrade", root:display = root.display().to_string(), error:display = e.to_string(); "failed to read root directory for DLL scanning");
            return Err(e.into());
        }
    }

    if dll_count > 0 {
        info!(SCOPE = "subcommands::upgrade", count = dll_count, plugins_dir:display = plugins_dir.display().to_string(); "installed DLL plugins");
        installation_count += 1;
    }

    if installation_count == 0 {
        debug!(SCOPE = "subcommands::upgrade", temp_dir:display = temp_dir.display().to_string(); "no mod components found to install - this may indicate wrapper folder issues");
    } else {
        info!(SCOPE = "subcommands::upgrade", components = installation_count, output_dir:display = output_dir.display().to_string(); "successfully installed mod components");
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() {
        create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            if let Some(parent) = target.parent() {
                create_dir_all(parent)?;
            }
            fs_copy(&path, &target)?; // overwrite
            normalize_permissions(&target);
        }
    }
    Ok(())
}

fn ensure_required_dirs(output_dir: &Path) -> Result<()> {
    let required = [
        output_dir.to_path_buf(),
        output_dir.join("BepInEx").join("plugins"),
        output_dir.join("user").join("mods"),
        output_dir.join("MODS"),
    ];
    for dir in required {
        if !dir.exists() {
            match create_dir_all(&dir) {
                Ok(_) => {
                    info!(SCOPE = "subcommands::upgrade", path:display = dir.display().to_string(); "created required directory")
                }
                Err(e) => println!(
                    "{} Failed to create directory {}: {}",
                    CROSS.red(),
                    dir.display(),
                    e
                ),
            }
        }
    }
    Ok(())
}

/// Get the latest compatible downloadable for the mods in `profile`
///
/// If an error occurs with a resolving task, instead of failing immediately,
/// resolution will continue and the error return flag is set to true.
pub async fn get_platform_downloadables(profile: &Profile) -> Result<(Vec<DownloadData>, bool)> {
    // let progress_bar = Arc::new(Mutex::new(ProgressBar::new(0).with_style(STYLE_NO.clone())));
    // Progress bar temporarily disabled
    let mut tasks = JoinSet::new();

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    // progress_bar
    //     .lock()
    //     .enable_steady_tick(Duration::from_millis(100));
    let pad_len = profile
        .mods
        .iter()
        .map(|m| m.name.len())
        .max()
        .unwrap_or(20)
        .clamp(20, 50);

    // Spawn a task per mod (dependency expansion can be re-added later if needed)
    for mod_ in profile.mods.clone() {
        // progress_bar.lock().inc_length(1);
        let filters = profile.filters.clone();
        // let progress_bar = Arc::clone(&progress_bar);
        tasks.spawn(async move {
            let permit = SEMAPHORE.get_or_init(default_semaphore).acquire().await?;
            let result = mod_.fetch_download_file(filters).await;
            drop(permit);

            // progress_bar.lock().inc(1);
            match result {
                Ok(download_file) => {
                    println!(
                        "{} {:pad_len$}  {}",
                        TICK.clone(),
                        mod_.name,
                        download_file.filename().dimmed()
                    );
                    Ok(Some(download_file))
                }
                Err(err) => {
                    println!("{}", format!("{CROSS} {:pad_len$}  {err}", mod_.name).red());
                    Ok(None)
                }
            }
        });
    }

    // Wait for all tasks to finish before clearing the bar
    let task_results = tasks
        .join_all()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    // progress_bar.lock().finish_and_clear();

    let error = task_results.iter().any(Option::is_none);
    let to_download = task_results.into_iter().flatten().collect();
    Ok((to_download, error))
}

pub async fn upgrade(profile: &mut Profile, local_only: bool) -> Result<()> {
    ensure_required_dirs(&profile.output_dir)?;

    if local_only {
        info!(SCOPE = "subcommands::upgrade", output_dir:display = profile.output_dir.display().to_string(); "running upgrade in local-only mode, scanning MODS directory");

        // Copy archives from MODS directory to output directory for processing
        let mods_dir = profile.output_dir.join("MODS");
        if !mods_dir.exists() {
            println!(
                "{}",
                "No MODS directory found - nothing to install locally".yellow()
            );
            return Ok(());
        }

        let mut archive_count = 0;
        for entry in read_dir(&mods_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    match ext.to_ascii_lowercase().as_str() {
                        "zip" | "7z" => {
                            let target = profile.output_dir.join(path.file_name().unwrap());
                            if !target.exists() {
                                info!(SCOPE = "subcommands::upgrade", from:display = path.display().to_string(), to:display = target.display().to_string(); "copying archive from MODS for local installation");
                                fs_copy(&path, &target)?;
                                archive_count += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if archive_count == 0 {
            println!("{}", "No archives found in MODS directory".yellow());
        } else {
            println!(
                "\n{} {} archives from MODS directory",
                "Found".bold(),
                archive_count
            );
        }

        // Extract all archives (both existing and copied from MODS)
        if let Err(e) = extract_all_archives(&profile.output_dir) {
            println!("{} Failed to extract some archives: {}", CROSS.red(), e);
        }

        Ok(())
    } else {
        let (mut to_download, error) = get_platform_downloadables(profile).await?;
        let mut to_install = Vec::new();
        if profile.output_dir.join("user").exists() {
            for file in read_dir(profile.output_dir.join("user"))? {
                let file = file?;
                let path = file.path();
                if path.is_file() {
                    to_install.push((file.file_name(), path));
                }
            }
        }

        clean(&profile.output_dir, &mut to_download, &mut to_install).await?;
        to_download
            .iter_mut()
            // Download directly to the output directory
            .map(|thing| thing.output = thing.filename().into())
            .for_each(drop); // Doesn't drop any data, just runs the iterator
                             // Always attempt extraction of any archives present (new or existing)
        if to_download.is_empty() && to_install.is_empty() {
            println!("\n{}", "All up to date!".bold());
            if let Err(e) = extract_all_archives(&profile.output_dir) {
                println!("{} Failed to extract some archives: {}", CROSS.red(), e);
            }
        } else {
            println!("\n{}\n", "Downloading Mod Files".bold());
            download(profile.output_dir.clone(), to_download, to_install).await?;
            if let Err(e) = extract_all_archives(&profile.output_dir) {
                println!("{} Failed to extract some archives: {}", CROSS.red(), e);
            }
        }

        // Update mod files tracking after upgrade
        if let Err(e) = update_mod_files(profile) {
            println!("{} Failed to update mod files tracking: {}", "Warning:".yellow(), e);
        }

        if error {
            Err(anyhow!(
                "\nCould not get the latest compatible version of some mods"
            ))
        } else {
            Ok(())
        }
    }
}
