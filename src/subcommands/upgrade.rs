use crate::{
    default_semaphore,
    download::{clean, download},
    CROSS, SEMAPHORE, STYLE_NO, TICK,
};
use anyhow::{anyhow, bail, Result};
use colored::Colorize as _;
use indicatif::ProgressBar;
use libarov::{
    config::{
        filters::ProfileParameters as _,
        structs::{Mod, ModIdentifier, Profile},
    },
    upgrade::{mod_downloadable, DownloadData},
};
use parking_lot::Mutex;
use std::{
    fs::read_dir,
    mem::take,
    sync::Arc,
    time::Duration,
};
use tokio::task::JoinSet;
use std::{fs::{self, File, create_dir_all, copy as fs_copy}, path::{Path, PathBuf}};
use zip::ZipArchive;
use sevenz_rust::decompress_file;

fn extract_all_archives(output_dir: &Path) -> Result<()> {
    for entry in fs::read_dir(output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                match ext.to_ascii_lowercase().as_str() {
                    "zip" => {
                        if let Err(e) = extract_zip(&path, output_dir) {
                            println!("{} Failed extracting {}: {}", CROSS.red(), path.file_name().unwrap_or_default().to_string_lossy(), e);
                        } else {
                            println!("{} Extracted        {}", TICK.clone(), path.file_name().unwrap_or_default().to_string_lossy().dimmed());
                        }
                    }
                    "7z" => {
                        if let Err(e) = extract_7z(&path, output_dir) {
                            println!("{} Failed extracting {}: {}", CROSS.red(), path.file_name().unwrap_or_default().to_string_lossy(), e);
                        } else {
                            println!("{} Extracted        {}", TICK.clone(), path.file_name().unwrap_or_default().to_string_lossy().dimmed());
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn extract_7z(archive_path: &Path, output_dir: &Path) -> Result<()> {
    let temp_dir = output_dir.join(".extract_tmp").join(archive_path.file_stem().unwrap_or_default());
    if temp_dir.exists() { fs::remove_dir_all(&temp_dir)?; }
    create_dir_all(&temp_dir)?;
    decompress_file(archive_path, &temp_dir)?;
    install_extracted(&temp_dir, output_dir)?;
    fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

fn extract_zip(zip_path: &Path, output_dir: &Path) -> Result<()> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let temp_dir = output_dir.join(".extract_tmp").join(zip_path.file_stem().unwrap_or_default());
    if temp_dir.exists() { fs::remove_dir_all(&temp_dir)?; }
    create_dir_all(&temp_dir)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = temp_dir.join(file.mangled_name());
        if file.is_dir() {
            create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() { create_dir_all(parent)?; }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    install_extracted(&temp_dir, output_dir)?;
    fs::remove_dir_all(&temp_dir)?;
    Ok(())
}

fn install_extracted(temp_dir: &Path, output_dir: &Path) -> Result<()> {
    // Collapse single-folder wrappers
    let mut root = temp_dir.to_path_buf();
    if let Ok(entries) = fs::read_dir(&root) {
        let collected: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        if collected.len() == 1 && collected[0].file_type().map(|t| t.is_dir()).unwrap_or(false) {
            root = collected[0].path();
        }
    }

    // Merge BepInEx if present
    let bep = root.join("BepInEx");
    if bep.exists() {
        copy_dir_recursive(&bep, &output_dir.join("BepInEx"))?;
    }
    // Merge user mods
    let user_dir = root.join("user");
    if user_dir.exists() {
        copy_dir_recursive(&user_dir, &output_dir.join("user"))?;
    }
    // Top-level dlls => BepInEx/plugins
    let plugins_dir = output_dir.join("BepInEx").join("plugins");
    create_dir_all(&plugins_dir)?;
    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("dll")).unwrap_or(false) {
                let target = plugins_dir.join(p.file_name().unwrap());
                fs_copy(&p, &target)?;
            }
        }
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !dst.exists() { create_dir_all(dst)?; }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            if let Some(parent) = target.parent() { create_dir_all(parent)?; }
            fs_copy(&path, &target)?; // overwrite
        }
    }
    Ok(())
}

/// Get the latest compatible downloadable for the mods in `profile`
///
/// If an error occurs with a resolving task, instead of failing immediately,
/// resolution will continue and the error return flag is set to true.
pub async fn get_platform_downloadables(profile: &Profile) -> Result<(Vec<DownloadData>, bool)> {
    let progress_bar = Arc::new(Mutex::new(ProgressBar::new(0).with_style(STYLE_NO.clone())));
    let mut tasks = JoinSet::new();

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    progress_bar
        .lock()
        .enable_steady_tick(Duration::from_millis(100));
    let pad_len = profile
        .mods
        .iter()
        .map(|m| m.name.len())
        .max()
        .unwrap_or(20)
        .clamp(20, 50);

    // Spawn a task per mod (dependency expansion can be re-added later if needed)
    for mod_ in profile.mods.clone() {
        progress_bar.lock().inc_length(1);
        let filters = profile.filters.clone();
        let progress_bar = Arc::clone(&progress_bar);
        tasks.spawn(async move {
            let permit = SEMAPHORE.get_or_init(default_semaphore).acquire().await?;
            let result = mod_.fetch_download_file(filters).await;
            drop(permit);

            progress_bar.lock().inc(1);
            match result {
                Ok(download_file) => {
                    progress_bar.lock().println(format!(
                        "{} {:pad_len$}  {}",
                        TICK.clone(),
                        mod_.name,
                        download_file.filename().dimmed()
                    ));
                    Ok(Some(download_file))
                }
                Err(err) => {
                    progress_bar.lock().println(format!(
                        "{}",
                        format!("{CROSS} {:pad_len$}  {err}", mod_.name).red()
                    ));
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

    progress_bar.lock().finish_and_clear();

    let error = task_results.iter().any(Option::is_none);
    let to_download = task_results.into_iter().flatten().collect();
    Ok((to_download, error))
}

pub async fn upgrade(profile: &Profile) -> Result<()> {
    let (mut to_download, error) = get_platform_downloadables(profile).await?;
    let mut to_install = Vec::new();
    if profile.output_dir.join("user").exists() {
        for file in read_dir(profile.output_dir.join("user"))? {
            let file = file?;
            let path = file.path();
            if path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
            {
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
    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!("\n{}\n", "Downloading Mod Files".bold());
        download(profile.output_dir.clone(), to_download, to_install).await?;
        // After downloading archives, extract them into SPT structure
        if let Err(e) = extract_all_archives(&profile.output_dir) {
            println!("{} Failed to extract some archives: {}", CROSS.red(), e);
        }
    }

    if error {
        Err(anyhow!(
            "\nCould not get the latest compatible version of some mods"
        ))
    } else {
        Ok(())
    }
}
