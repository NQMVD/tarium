use anyhow::{bail, Result};
use colored::Colorize as _;
use inquire::MultiSelect;
use libarov::{
    config::{mod_state::ModStateManager, structs::ModIdentifier, structs::Profile},
    iter_ext::IterExt as _,
};

/// Enable mods that were previously disabled
pub fn enable_mods(profile: &mut Profile, to_enable: Vec<String>) -> Result<()> {
    let mod_state_manager = ModStateManager::new(profile.output_dir.clone());
    
    let indices_to_enable = if to_enable.is_empty() {
        // Show only disabled mods for selection
        let disabled_mods: Vec<_> = profile.mods.iter()
            .filter(|mod_| !mod_.enabled)
            .map(|mod_| {
                format!(
                    "{:11}  {}",
                    match &mod_.identifier {
                        ModIdentifier::GitHubRepository(..) => "GH".to_string(),
                        _ => todo!(),
                    },
                    match &mod_.identifier {
                        ModIdentifier::GitHubRepository(owner, repo) => format!("{}/{}", owner.dimmed(), repo.bold()),
                        _ => todo!(),
                    },
                )
            })
            .collect_vec();
        
        if disabled_mods.is_empty() {
            println!("{}", "No disabled mods found to enable".yellow());
            return Ok(());
        }
        
        let selections = MultiSelect::new("Select mods to enable", disabled_mods.clone())
            .raw_prompt_skippable()?
            .unwrap_or_default();
        
        // Map selected indices back to original mod indices
        let disabled_mod_indices: Vec<usize> = profile.mods.iter()
            .enumerate()
            .filter(|(_, mod_)| !mod_.enabled)
            .map(|(idx, _)| idx)
            .collect_vec();
        
        selections.iter().map(|o| disabled_mod_indices[o.index]).collect_vec()
    } else {
        let mut items_to_enable = Vec::new();
        for to_enable in to_enable {
            if let Some(index) = profile.mods.iter().position(|mod_| {
                mod_.name.eq_ignore_ascii_case(&to_enable)
                    || match &mod_.identifier {
                        ModIdentifier::GitHubRepository(owner, name) => {
                            format!("{owner}/{name}").eq_ignore_ascii_case(&to_enable)
                        }
                        _ => todo!(),
                    }
                    || mod_
                        .slug
                        .as_ref()
                        .is_some_and(|slug| to_enable.eq_ignore_ascii_case(slug))
            }) {
                if profile.mods[index].enabled {
                    println!("{} {} is already enabled", "Warning:".yellow(), profile.mods[index].name.bold());
                    continue;
                }
                items_to_enable.push(index);
            } else {
                bail!("A mod with ID or name {} is not present in this profile", to_enable);
            }
        }
        items_to_enable
    };

    if indices_to_enable.is_empty() {
        println!("{}", "No mods selected for enabling".yellow());
        return Ok(());
    }

    let mut enabled = Vec::new();
    for index in indices_to_enable {
        let mod_ = &mut profile.mods[index];
        
        // Move files from disabled to enabled directory
        mod_state_manager.enable_mod(&mod_.name, &mod_.files)?;
        
        mod_.enabled = true;
        enabled.push(mod_.name.clone());
    }

    if !enabled.is_empty() {
        println!(
            "Enabled {}",
            enabled.iter().map(|txt| txt.bold().green()).display(", ")
        );
    }

    Ok(())
}

/// Disable mods without removing them from the profile
pub fn disable_mods(profile: &mut Profile, to_disable: Vec<String>) -> Result<()> {
    let mod_state_manager = ModStateManager::new(profile.output_dir.clone());
    
    let indices_to_disable = if to_disable.is_empty() {
        // Show only enabled mods for selection
        let enabled_mods: Vec<_> = profile.mods.iter()
            .filter(|mod_| mod_.enabled)
            .map(|mod_| {
                format!(
                    "{:11}  {}",
                    match &mod_.identifier {
                        ModIdentifier::GitHubRepository(..) => "GH".to_string(),
                        _ => todo!(),
                    },
                    match &mod_.identifier {
                        ModIdentifier::GitHubRepository(owner, repo) => format!("{}/{}", owner.dimmed(), repo.bold()),
                        _ => todo!(),
                    },
                )
            })
            .collect_vec();
        
        if enabled_mods.is_empty() {
            println!("{}", "No enabled mods found to disable".yellow());
            return Ok(());
        }
        
        let selections = MultiSelect::new("Select mods to disable", enabled_mods.clone())
            .raw_prompt_skippable()?
            .unwrap_or_default();
        
        // Map selected indices back to original mod indices
        let enabled_mod_indices: Vec<usize> = profile.mods.iter()
            .enumerate()
            .filter(|(_, mod_)| mod_.enabled)
            .map(|(idx, _)| idx)
            .collect_vec();
        
        selections.iter().map(|o| enabled_mod_indices[o.index]).collect_vec()
    } else {
        let mut items_to_disable = Vec::new();
        for to_disable in to_disable {
            if let Some(index) = profile.mods.iter().position(|mod_| {
                mod_.name.eq_ignore_ascii_case(&to_disable)
                    || match &mod_.identifier {
                        ModIdentifier::GitHubRepository(owner, name) => {
                            format!("{owner}/{name}").eq_ignore_ascii_case(&to_disable)
                        }
                        _ => todo!(),
                    }
                    || mod_
                        .slug
                        .as_ref()
                        .is_some_and(|slug| to_disable.eq_ignore_ascii_case(slug))
            }) {
                if !profile.mods[index].enabled {
                    println!("{} {} is already disabled", "Warning:".yellow(), profile.mods[index].name.bold());
                    continue;
                }
                items_to_disable.push(index);
            } else {
                bail!("A mod with ID or name {} is not present in this profile", to_disable);
            }
        }
        items_to_disable
    };

    if indices_to_disable.is_empty() {
        println!("{}", "No mods selected for disabling".yellow());
        return Ok(());
    }

    let mut disabled = Vec::new();
    for index in indices_to_disable {
        let mod_ = &mut profile.mods[index];
        
        // Move files from enabled to disabled directory
        mod_state_manager.disable_mod(&mod_.name, &mod_.files)?;
        
        mod_.enabled = false;
        disabled.push(mod_.name.clone());
    }

    if !disabled.is_empty() {
        println!(
            "Disabled {}",
            disabled.iter().map(|txt| txt.bold().red()).display(", ")
        );
    }

    Ok(())
}