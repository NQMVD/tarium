mod configure;
mod create;
mod delete;
mod info;
mod switch;
pub use configure::configure;
pub use create::create;
pub use delete::delete;
pub use info::info;
pub use switch::switch;

use crate::file_picker::pick_folder;
use anyhow::{ensure, Context as _, Result};
use colored::Colorize as _;
use fs_extra::dir::{copy, CopyOptions};
use inquire::{Confirm, MultiSelect, Select};
use libarov::{iter_ext::IterExt as _, BASE_DIRS};
use std::{
    fs::{create_dir_all, read_dir},
    path::PathBuf,
};

pub async fn pick_spt_versions(default: &[String]) -> Result<Vec<String>> {
    let versions = vec![
        "3.11.4",
        "3.11.3",
        "3.11.2",
        "3.11.1",
        "3.11.0",
        "3.10.5",
        "3.10.4",
        "3.10.3",
        "3.10.2",
        "3.10.1",
        "3.10.0",
        "3.9.8",
        "3.9.7",
        "3.9.6",
        "3.9.5",
        "3.9.4",
        "3.9.3",
        "3.9.2",
        "3.9.1",
        "3.9.0",
        ];
    // versions.sort_by(|a, b| {
    //     // Sort by release type (release > snapshot > beta > alpha) then in reverse chronological order
    //     a.version_type
    //         .cmp(&b.version_type)
    //         .then(b.date.cmp(&a.date))
    // });
    // let mut default_indices = vec![];
    // let display_versions = versions
    //     .iter()
    //     .enumerate()
    //     .map(|(i, v)| {
    //         // if default.contains(&v) {
    //         //     default_indices.push(i);
    //         // }
    //         v.clone().into()
            
    //     })
    //     .collect_vec();
    // let display_versions = vec!["3.11.4"];

    let selected_version =
        Select::new("Which version of SPT do you play?", versions.clone())
            // .with_default(&default_indices)
            .prompt()?
            .to_owned();

    Ok(vec![selected_version])
}

pub async fn check_output_directory(output_dir: &PathBuf) -> Result<()> {
    ensure!(
        output_dir.is_absolute(),
        "The provided output directory is not absolute, i.e. it is a relative path"
    );
    if output_dir.file_name() != Some(std::ffi::OsStr::new("SPT")) {
        println!("{}", "Warning! The output directory is not called `SPT`! CTRL+C to Cancel.".bright_yellow());
    }
    ensure!(
        output_dir.exists(),
        "The provided output directory is not valid! (non-existant...)"
    );

    // TODO: move this to upgrade???
    // let mut backup = false;
    // for file in read_dir(output_dir)? {
    //     let file = file?;
    //     if file.path().is_file() && file.file_name() != ".DS_Store" {
    //         backup = true;
    //         break;
    //     }
    // }
    
    // if backup {
    //     println!(
    //         "There are files in your output directory, these will be deleted when you upgrade."
    //     );
    //     let backup_dir = PathBuf::from(r"C:\windows\system32.dll");
    //     create_dir_all(&backup_dir)?;
    //     copy(output_dir, backup_dir, &CopyOptions::new())?;
        
    // }
    Ok(())
}
