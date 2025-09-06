#![deny(missing_docs)]

use crate::DEFAULT_PARALLEL_TASKS;
use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};
use clap_complete::Shell;
use libarov::config::filters::{self, Filter};
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about)]
pub struct Tarium {
    #[clap(subcommand)]
    pub subcommand: SubCommands,
    /// Sets the number of worker threads the tokio runtime will use.
    /// You can also use the environment variable `TOKIO_WORKER_THREADS`.
    #[clap(long, short)]
    pub threads: Option<usize>,
    /// Specify the maximum number of simultaneous parallel tasks.
    #[clap(long, short = 'p', default_value_t = DEFAULT_PARALLEL_TASKS)]
    pub parallel_tasks: usize,
    /// Increase output verbosity (-v, -vv, -vvv, etc.)
    #[clap(long, short = 'v', action = clap::ArgAction::Count)]
    pub verbosity: u8,
    /// Set a GitHub personal access token for increasing the GitHub API rate limit.
    #[clap(long, visible_alias = "gh", env = "GITHUB_TOKEN")]
    pub github_token: Option<String>,
    /// Set the file to read the config from.
    /// This does not change the `cache` and `tmp` directories.
    /// You can also use the environment variable `TARIUM_CONFIG_FILE`.
    #[clap(long, short, visible_aliases = ["config", "conf"])]
    #[clap(value_hint(ValueHint::FilePath))]
    pub config_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SubCommands {
    /*  TODO:
        Use this for filter arguments:
        https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_3/index.html#argument-relations
    */
    /// Add mods to the profile
    Add {
        /// The identifier(s) of the repository
        ///
        /// The GitHub identifier is the repository's full name, e.g. `Solarint/SAIN`.
        #[clap(required = true)]
        identifiers: Vec<String>,

        /// Temporarily ignore game version and mod loader checks and add the mod anyway
        #[clap(long, short, visible_alias = "no-checks")]
        force: bool,

        /// Pin a mod to a specific version - CURRENTLY DISABLED
        #[clap(long, short, visible_alias = "lock")]
        pin: Option<String>,
    },
    /// Configure GitHub App authentication for higher API rate limits
    Auth {
        #[clap(subcommand)]
        subcommand: AuthSubCommands,
    },
    /// Add mods to the profile from a file containing a list of identifiers
    AddFrom {
        /// The file containing mod identifiers (one per line)
        ///
        /// Each line should contain a GitHub repository identifier in the format `owner/repo`.
        /// Empty lines and lines starting with # are ignored.
        #[clap(value_hint(ValueHint::FilePath))]
        file: PathBuf,

        /// Temporarily ignore game version and mod loader checks and add the mod anyway
        #[clap(long, short, visible_alias = "no-checks")]
        force: bool,
    },
    /// Print shell auto completions for the specified shell
    Complete {
        /// The shell to generate auto completions for
        #[clap(value_enum)]
        shell: Shell,
    },
    /// List all the mods in the profile, and with some their metadata if verbose
    #[clap(visible_alias = "mods")]
    List {
        /// Show additional information about the mod
        #[clap(long, short)]
        verbose: bool,
        /// Output information in markdown format and alphabetical order
        ///
        /// Useful for creating modpack mod lists.
        /// Complements the verbose flag.
        #[clap(long, short, visible_alias = "md")]
        markdown: bool,
    },
    /// Create, configure, delete, switch, or list profiles
    Profile {
        #[clap(subcommand)]
        subcommand: Option<ProfileSubCommands>,
    },
    /// List all the profiles with their data
    Profiles,
    /// Remove mods and/or repositories from the profile.
    /// Optionally, provide a list of names or IDs of the mods to remove.
    #[clap(visible_aliases = ["rm", "delete"])]
    Remove {
        /// List of project IDs or case-insensitive names of mods to remove
        mod_names: Vec<String>,
    },
    /// Download and install the latest compatible version of your mods
    #[clap(visible_aliases = ["download", "install", "update"])]
    Upgrade {
        /// Skip downloading and only install mods already present in the MODS directory
        #[clap(long, short, visible_aliases = ["local", "offline", "no-download"])]
        local_only: bool,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum ProfileSubCommands {
    /// Configure the current profile's name, Minecraft version, mod loader, and output directory.
    /// Optionally, provide the settings to change as arguments.
    #[clap(visible_aliases = ["config", "conf"])]
    Configure {
        /// The Minecraft version(s) to consider as compatible
        #[clap(long, short = 'v')]
        game_versions: Vec<String>,
        /// The name of the profile
        #[clap(long, short)]
        name: Option<String>,
        /// The directory to output mods to
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        output_dir: Option<PathBuf>,
    },
    /// Create a new profile.
    /// Optionally, provide the settings as arguments.
    /// Use the import flag to import mods from another profile.
    #[clap(visible_alias = "new")]
    Create {
        /// Copy over the mods from an existing profile.
        /// Optionally, provide the name of the profile to import mods from.
        #[clap(long, short, visible_aliases = ["copy", "duplicate"])]
        #[expect(clippy::option_option)]
        import: Option<Option<String>>,
        /// The directory to output mods to
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        output_dir: Option<PathBuf>,
        /// The Minecraft version to check compatibility for
        #[clap(long, short = 'v')]
        game_version: Vec<String>,
        /// The name of the profile
        #[clap(long, short)]
        name: Option<String>,
    },
    /// Delete a profile.
    /// Optionally, provide the name of the profile to delete.
    #[clap(visible_aliases = ["remove", "rm"])]
    Delete {
        /// The name of the profile to delete
        profile_name: Option<String>,
        /// The name of the profile to switch to afterwards
        #[clap(long, short)]
        switch_to: Option<String>,
    },
    /// Show information about the current profile
    Info,
    /// List all the profiles with their data
    List,
    /// Switch between different profiles.
    /// Optionally, provide the name of the profile to switch to.
    Switch {
        /// The name of the profile to switch to
        profile_name: Option<String>,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum ModpackSubCommands {
    /// Add a modpack to the config
    Add {
        /// The identifier of the modpack/project
        ///
        /// The Modrinth project ID is specified at the bottom of the left sidebar under 'Technical information'.
        /// You can also use the project slug for this.
        /// The Curseforge project ID is specified at the top of the right sidebar under 'About Project'.
        identifier: String,
        /// The Minecraft instance directory to install the modpack to
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        output_dir: Option<PathBuf>,
        /// Whether to install the modpack's overrides to the output directory.
        /// This will override existing files when upgrading.
        #[clap(long, short)]
        install_overrides: Option<bool>,
    },
    /// Configure the current modpack's output directory and installation of overrides.
    /// Optionally, provide the settings to change as arguments.
    #[clap(visible_aliases = ["config", "conf"])]
    Configure {
        /// The Minecraft instance directory to install the modpack to
        #[clap(long, short)]
        #[clap(value_hint(ValueHint::DirPath))]
        output_dir: Option<PathBuf>,
        /// Whether to install the modpack's overrides to the output directory.
        /// This will override existing files when upgrading.
        #[clap(long, short)]
        install_overrides: Option<bool>,
    },
    /// Delete a modpack.
    /// Optionally, provide the name of the modpack to delete.
    #[clap(visible_aliases = ["remove", "rm"])]
    Delete {
        /// The name of the modpack to delete
        modpack_name: Option<String>,
        /// The name of the profile to switch to afterwards
        #[clap(long, short)]
        switch_to: Option<String>,
    },
    /// Show information about the current modpack
    Info,
    /// List all the modpacks with their data
    List,
    /// Switch between different modpacks.
    /// Optionally, provide the name of the modpack to switch to.
    Switch {
        /// The name of the modpack to switch to
        modpack_name: Option<String>,
    },
    /// Download and install the latest version of the modpack
    #[clap(visible_aliases = ["download", "install"])]
    Upgrade,
}

#[derive(Clone, Default, Debug, Args)]
#[group(id = "loader", multiple = false)]
pub struct FilterArguments {
    #[clap(long)]
    pub override_profile: bool,

    #[clap(long, short = 'v', group = "version")]
    pub game_version_strict: Vec<String>,
    #[clap(long, group = "version")]
    pub game_version_minor: Vec<String>,

    #[clap(long, short = 'c')]
    pub release_channel: Option<filters::ReleaseChannel>,

    #[clap(long, short = 'n')]
    pub filename: Option<String>,
    #[clap(long, short = 't')]
    pub title: Option<String>,
    #[clap(long, short = 'd')]
    pub description: Option<String>,
}

impl From<FilterArguments> for Vec<Filter> {
    fn from(value: FilterArguments) -> Self {
        let mut filters = vec![];

        if !value.game_version_strict.is_empty() {
            filters.push(Filter::GameVersionStrict(value.game_version_strict));
        }
        if !value.game_version_minor.is_empty() {
            filters.push(Filter::GameVersionMinor(value.game_version_minor));
        }
        if let Some(release_channel) = value.release_channel {
            filters.push(Filter::ReleaseChannel(release_channel));
        }
        if let Some(regex) = value.filename {
            filters.push(Filter::Filename(regex));
        }
        if let Some(regex) = value.title {
            filters.push(Filter::Title(regex));
        }
        if let Some(regex) = value.description {
            filters.push(Filter::Description(regex));
        }

        filters
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum AuthSubCommands {
    /// Set up GitHub App credentials (no user authentication required)
    Login {
        /// GitHub App ID (will prompt if not provided)
        #[clap(long)]
        client_id: Option<String>,
        /// Unused (kept for compatibility)
        #[clap(long, hide = true)]
        client_secret: Option<String>,
    },
    /// Remove stored GitHub App credentials
    Logout,
    /// Check GitHub App authentication status
    Status,
}
