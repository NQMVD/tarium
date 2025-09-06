#![deny(
    clippy::all,
    clippy::perf,
    clippy::cargo,
    clippy::style,
    clippy::pedantic,
    clippy::suspicious,
    clippy::complexity,
    clippy::create_dir,
    clippy::unwrap_used,
    clippy::correctness,
    clippy::allow_attributes
)]
#![deny(clippy::expect_used, reason = "Use anyhow::Context instead")]
#![warn(clippy::dbg_macro)]
#![expect(clippy::multiple_crate_versions, clippy::too_many_lines)]
#![cfg_attr(debug_assertions, allow(warnings))]

mod add;
mod auth;
mod cli;
mod download;
mod file_picker;
mod logging;
mod subcommands;

#[cfg(test)]
mod tests;

use anyhow::{anyhow, bail, ensure, Context as _, Result};
use clap::{CommandFactory, Parser};
use cli::{ProfileSubCommands, SubCommands, Tarium};
use colored::{ColoredString, Colorize};
use indicatif::ProgressStyle;
use libarov::{
    config::{
        self,
        filters::ProfileParameters as _,
        structs::{Config, ModIdentifier, Profile},
    },
    iter_ext::IterExt as _,
};
use log::{debug, info, warn};
use std::{
    env::{set_var, var_os},
    process::ExitCode,
    sync::{LazyLock, OnceLock},
};
use tokio::sync::Semaphore;

const CROSS: &str = "×";
static TICK: LazyLock<ColoredString> = LazyLock::new(|| "✓".green());

pub const DEFAULT_PARALLEL_TASKS: usize = 50;
pub static SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();
#[must_use]
pub const fn default_semaphore() -> Semaphore {
    Semaphore::const_new(DEFAULT_PARALLEL_TASKS)
}

/// Indicatif themes
#[expect(clippy::expect_used)]
pub static STYLE_NO: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::default_bar()
        .template("{spinner} {elapsed} [{wide_bar:.cyan/blue}] {pos:.cyan}/{len:.blue}")
        .expect("Progress bar template parse failure")
        .progress_chars("#>-")
});
#[expect(clippy::expect_used)]
pub static STYLE_BYTE: LazyLock<ProgressStyle> = LazyLock::new(|| {
    ProgressStyle::default_bar()
        .template(
            "{spinner} {bytes_per_sec} [{wide_bar:.cyan/blue}] {bytes:.cyan}/{total_bytes:.blue}",
        )
        .expect("Progress bar template parse failure")
        .progress_chars("#>-")
});

fn main() -> ExitCode {
    #[cfg(windows)]
    // Enable colours on conhost (command prompt or powershell)
    {
        #[expect(clippy::unwrap_used, reason = "There is actually no error")]
        colored::control::set_virtual_terminal(true).unwrap();
    }

    let cli = Tarium::parse();

    if let Err(e) = logging::setup_logger(cli.verbosity) {
        eprintln!("failed to init logger: {e}");
    } else {
        info!("logger initialised");
    }

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    builder.thread_name("tarium-worker");
    if let Some(threads) = cli.threads {
        builder.worker_threads(threads);
    }
    #[expect(clippy::expect_used)] // No error handling yet
    let runtime = builder.build().expect("Could not initialise Tokio runtime");

    if let Err(err) = runtime.block_on(actual_main(cli)) {
        if !err.to_string().is_empty() {
            eprintln!("{}", err.to_string().red().bold());
            if err
                .to_string()
                .to_lowercase()
                .contains("error trying to connect")
                || err
                    .to_string()
                    .to_lowercase()
                    .contains("error sending request")
            {
                eprintln!(
                    "{}",
                    "Verify that you are connected to the internet"
                        .yellow()
                        .bold()
                );
            }
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

async fn actual_main(mut cli_app: Tarium) -> Result<()> {
    // The complete command should not require a config.
    // See [#139](https://github.com/gorilla-devs/tarium/issues/139) for why this might be a problem.
    if let SubCommands::Complete { shell } = cli_app.subcommand {
        clap_complete::generate(
            shell,
            &mut Tarium::command(),
            "tarium",
            &mut std::io::stdout(),
        );
        return Ok(());
    }
    // Alias `tarium profiles` to `tarium profile list`
    if let SubCommands::Profiles = cli_app.subcommand {
        cli_app.subcommand = SubCommands::Profile {
            subcommand: Some(ProfileSubCommands::List),
        };
    }

    if let Some(token) = cli_app.github_token {
        if !token.is_empty() {
            set_var("GITHUB_TOKEN", token);
        }
    }

    let _ = SEMAPHORE.set(Semaphore::new(cli_app.parallel_tasks));

    let old_default_config_path = libarov::BASE_DIRS
        .home_dir()
        .join(".config")
        .join("tarium")
        .join("config.json");
    let config_path = &cli_app
        .config_file
        .or_else(|| var_os("TARIUM_CONFIG_FILE").map(Into::into))
        .unwrap_or({
            #[cfg(target_os = "macos")]
            {
                old_default_config_path.clone()
            }
            #[cfg(not(target_os = "macos"))]
            {
                libarov::PROJECT_DIRS.config_dir().join("config.json")
            }
        });
    info!(config_path:debug; "Resolved config path");

    // Handle old configs which may be in a different path

    if !config_path.exists() && old_default_config_path.exists() {
        info!(from:debug = &old_default_config_path, to:debug = config_path; "Relocating legacy config file");

        std::fs::rename(old_default_config_path, config_path)
            .context("Failed to relocate config file to the new path, try doing so manually.")?;

        info!(path:debug = config_path; "Legacy config file relocated");
    }

    let mut config = config::read_config(config_path)?;
    info!("Loaded config with {} profiles", config.profiles.len());

    // TODO: this needs a fucking rework holy shit
    let mut did_add_fail = false;

    // Run function(s) based on the sub(sub)command to be executed
    info!(SCOPE = "clap", subcommand:debug = cli_app.subcommand; "Executing");
    match cli_app.subcommand {
        SubCommands::Complete { .. } | SubCommands::Profiles => {
            unreachable!();
        }
        SubCommands::Auth { subcommand } => {
            subcommands::auth::handle_auth_command(subcommand).await?;
        }
        SubCommands::Add {
            identifiers,
            force,
            pin,
        } => {
            let profile = get_active_profile(&mut config)?;

            ensure!(
                // If a pin is specified, there should only be one mod
                pin.is_none() || identifiers.len() == 1,
                "You can only pin a version when adding a single mod!"
            );

            let identifiers = if let Some(pin) = pin {
                let id = libarov::add::parse_id(identifiers[0].clone());
                vec![match id {
                    ModIdentifier::GitHubRepository(owner, repo) => {
                        ModIdentifier::PinnedGitHubRepository(
                            (owner, repo),
                            pin.parse().context("Invalid asset ID for GitHub")?,
                        )
                    }
                    _ => unreachable!(),
                }]
            } else {
                identifiers
                    .into_iter()
                    .map(libarov::add::parse_id)
                    .collect_vec()
            };

            let (successes, failures) = libarov::add(profile, identifiers, !force).await?;

            did_add_fail = add::display_successes_failures(&successes, failures);
        }
        SubCommands::AddFrom { file, force } => {
            let profile = get_active_profile(&mut config)?;

            // Read the file and parse identifiers
            let file_content = std::fs::read_to_string(&file)
                .with_context(|| format!("Failed to read file: {}", file.display()))?;

            let mut identifiers = Vec::new();
            for line in file_content.lines().map(str::trim) {
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if !line.contains('/') {
                    bail!(
                        "Invalid identifier format: '{}'. Expected format: 'owner/repo'",
                        line
                    );
                }

                identifiers.push(libarov::add::parse_id(line.to_string()));
            }

            if identifiers.is_empty() {
                bail!("No valid mod identifiers found in file: {}", file.display());
            }

            info!(
                "Adding {} mods from file: {}",
                identifiers.len(),
                file.display()
            );

            let (successes, failures) = libarov::add(profile, identifiers, !force).await?;

            did_add_fail = add::display_successes_failures(&successes, failures);
        }
        SubCommands::List { verbose, markdown } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;

            if verbose {
                subcommands::list::verbose(profile, markdown).await?;
            } else {
                println!(
                    "{} {} on {}\n",
                    profile.name.bold(),
                    format!("({} mods)", profile.mods.len()).yellow(),
                    profile
                        .filters
                        .game_versions()
                        .unwrap_or(&vec![])
                        .iter()
                        .display(", ")
                        .green(),
                );
                for mod_ in &profile.mods {
                    println!(
                        "{:20}  {}",
                        match &mod_.identifier {
                            ModIdentifier::GitHubRepository(..) => "GH".purple().to_string(),
                            _ => todo!(),
                        },
                        match &mod_.identifier {
                            ModIdentifier::GitHubRepository(owner, repo) =>
                                format!("{}/{}", owner.dimmed(), repo.bold()),
                            _ => todo!(),
                        },
                    );
                }
            }
        }
        SubCommands::Profile { subcommand } => {
            let mut default_flag = false;
            let subcommand = subcommand.unwrap_or_else(|| {
                default_flag = true;
                ProfileSubCommands::Info
            });
            match subcommand {
                ProfileSubCommands::Configure {
                    game_versions,
                    name,
                    output_dir,
                } => {
                    subcommands::profile::configure(
                        get_active_profile(&mut config)?,
                        game_versions,
                        name,
                        output_dir,
                    )
                    .await?;
                }
                ProfileSubCommands::Create {
                    import,
                    output_dir,
                    game_version,
                    name,
                } => {
                    subcommands::profile::create(
                        &mut config,
                        output_dir,
                        import,
                        if game_version.is_empty() {
                            None
                        } else {
                            Some(game_version)
                        },
                        name,
                    )
                    .await?;
                }
                ProfileSubCommands::Delete {
                    profile_name,
                    switch_to,
                } => {
                    subcommands::profile::delete(&mut config, profile_name, switch_to)?;
                }
                ProfileSubCommands::Info => {
                    subcommands::profile::info(get_active_profile(&mut config)?, true);
                }

                ProfileSubCommands::List => {
                    for (i, profile) in config.profiles.iter().enumerate() {
                        subcommands::profile::info(profile, i == config.active_profile);
                    }
                }

                ProfileSubCommands::Switch { profile_name } => {
                    subcommands::profile::switch(&mut config, profile_name)?;
                }
            }
            if default_flag {
                println!(
                    "{} tarium profile help {}",
                    "Use".yellow(),
                    "for more information about this subcommand".yellow()
                );
            }
        }
        SubCommands::Remove { mod_names } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;
            subcommands::remove(profile, mod_names)?;
        }
        SubCommands::Upgrade { local_only } => {
            let profile = get_active_profile(&mut config)?;
            check_empty_profile(profile)?;
            subcommands::upgrade(profile, local_only).await?;
        }
    }

    config.profiles.iter_mut().for_each(|profile| {
        profile
            .mods
            .sort_unstable_by_key(|mod_| mod_.name.to_lowercase());
    });
    // Update config file with possibly edited config
    info!("Persisting config changes to {:?}", config_path);
    config::write_config(config_path, &config)?;

    if did_add_fail {
        Err(anyhow!("says did_add_fail here i guess..."))
    } else {
        Ok(())
    }
}

/// Get the active profile with error handling
fn get_active_profile(config: &mut Config) -> Result<&mut Profile> {
    match config.profiles.len() {
        0 => {
            bail!("There are no profiles configured, add a profile using `tarium profile create`")
        }
        1 => config.active_profile = 0,
        n if config.active_profile >= n => {
            warn!("Active profile index out of bounds, prompting switch");
            println!(
                "{}",
                "Active profile specified incorrectly, please pick a profile to use"
                    .red()
                    .bold()
            );
            subcommands::profile::switch(config, None)?;
        }
        _ => (),
    }
    Ok(&mut config.profiles[config.active_profile])
}

/// Check if `profile` is empty, and if so return an error
fn check_empty_profile(profile: &Profile) -> Result<()> {
    ensure!(
        !profile.mods.is_empty(),
        "Your currently selected profile is empty! Run `tarium help` to see how to add mods"
    );
    Ok(())
}
