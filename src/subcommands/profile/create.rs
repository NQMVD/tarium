use super::{check_output_directory, pick_spt_versions};
use crate::file_picker::pick_folder;
use anyhow::{bail, ensure, Context as _, Result};
use colored::Colorize as _;
use inquire::{
    validator::{ErrorMessage, Validation},
    Confirm, Select, Text,
};
use libarov::{
    config::{filters::Filter, structs::{Config, Profile}},
    get_spt_dir,
    iter_ext::IterExt as _,
};
use std::path::PathBuf;

#[expect(clippy::option_option)]
pub async fn create(
    config: &mut Config,
    output_dir: Option<PathBuf>,
    import: Option<Option<String>>,
    game_versions: Option<Vec<String>>,
    name: Option<String>,
) -> Result<()> {
    let mut profile = match (game_versions, name, output_dir) {
        (Some(game_versions), Some(name), Some(output_dir)) => {
            for profile in &config.profiles {
                ensure!(
                    !profile.name.eq_ignore_ascii_case(&name),
                    "A profile with name {name} already exists"
                );
            }
            let output_dir = output_dir;
            ensure!(
                output_dir.is_absolute(),
                "The provided output directory is not absolute, i.e. it is a relative path"
            );

            Profile::new(name, output_dir, game_versions, false)
        }
        (None, None, None) => {
            let mut selected_mods_dir = PathBuf::new();
            if let Some(dir) = pick_folder(
                &selected_mods_dir,
                "Pick an output directory",
                "Output Directory",
            )? {
                check_output_directory(&dir).await?;
                selected_mods_dir = dir;
            }
            
            let profiles = config.profiles.clone();
            let name = Text::new("What should this profile be called?")
                .with_validator(move |s: &str| {
                    Ok(if profiles.iter().any(|p| p.name.eq_ignore_ascii_case(s)) {
                        Validation::Invalid(ErrorMessage::Custom(
                            "A profile with that name already exists".to_owned(),
                        ))
                    } else {
                        Validation::Valid
                    })
                })
                .prompt()?;

            Profile::new(
                name,
                selected_mods_dir,
                pick_spt_versions(&[]).await?,
                false,
            )
        }
        _ => {
            bail!("Provide the name, game version, and output directory options to create a profile")
        }
    };

    if let Some(from) = import {
        ensure!(
            !config.profiles.is_empty(),
            "There are no profiles configured to import mods from"
        );

        // If the profile name has been provided as an option
        if let Some(profile_name) = from {
            let selection = config
                .profiles
                .iter()
                .position(|profile| profile.name.eq_ignore_ascii_case(&profile_name))
                .context("The profile name provided does not exist")?;
            profile.mods.clone_from(&config.profiles[selection].mods);
        } else {
            let profile_names = config
                .profiles
                .iter()
                .map(|profile| &profile.name)
                .collect_vec();
            if let Ok(selection) =
                Select::new("Select which profile to import mods from", profile_names)
                    .with_starting_cursor(config.active_profile)
                    .raw_prompt()
            {
                profile
                    .mods
                    .clone_from(&config.profiles[selection.index].mods);
            }
        };
    }

    println!(
        "{}",
        "Done!".green()
    );
    println!(
        "{}",
        "After adding your mods, remember to run `ferium upgrade` to download them!".yellow()
    );

    config.profiles.push(profile);
    config.active_profile = config.profiles.len() - 1; // Make created profile active
    Ok(())
}
