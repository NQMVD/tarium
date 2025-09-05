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
    }

    if error {
        Err(anyhow!(
            "\nCould not get the latest compatible version of some mods"
        ))
    } else {
        Ok(())
    }
}
