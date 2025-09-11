#![cfg_attr(debug_assertions, allow(warnings))]

pub mod add;
pub mod archive_analyzer;
pub mod config;
pub mod iter_ext;
pub mod upgrade;
pub mod mod_state;

pub use add::add;

use directories::{BaseDirs, ProjectDirs};
use std::{path::PathBuf, sync::LazyLock};

use log::debug;
use regex::Regex;

pub static GITHUB_API: LazyLock<octocrab::Octocrab> = LazyLock::new(|| {
    let mut github = octocrab::OctocrabBuilder::new();

    // Try to get GitHub App token
    if let Some(token) = get_github_app_token_blocking() {
        github = github.personal_token(token);
    } else if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        github = github.personal_token(token);
    }

    github.build().expect("Could not build GitHub client")
});

pub static VERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?xi)
        \b
        v?                                  # optional leading v
        (?P<major>\d+)                      # major
        (?:\.(?P<minor>\d+|[xX\*]))?        # optional minor or wildcard
        (?:\.(?P<patch>\d+|[xX\*]))?        # optional patch or wildcard
        \b
    ",
    )
    .unwrap()
});

fn get_github_app_token_blocking() -> Option<String> {
    // This will be set by the main crate
    std::env::var("TARIUM_GITHUB_APP_TOKEN").ok()
}

pub static BASE_DIRS: LazyLock<BaseDirs> =
    LazyLock::new(|| BaseDirs::new().expect("Could not get OS specific directories"));

pub static PROJECT_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    if cfg!(debug_assertions) {
        ProjectDirs::from("", "", "tarium-dev").expect("Could not get OS specific directories")
    } else {
        ProjectDirs::from("", "", "tarium").expect("Could not get OS specific directories")
    }
});

/// Gets the default SPT directory based on the current compilation `target_os`
pub fn get_spt_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        BASE_DIRS.data_dir().join("spt")
    }
    #[cfg(target_os = "windows")]
    {
        BASE_DIRS.data_dir().join(".spt-tarium")
        // TODO: get current dir and check if its name is SPT
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        BASE_DIRS.home_dir().join(".spt")
    }
}

/// Read `source` and return the data as a string
///
/// A wrapper for dealing with the read buffer.
pub fn read_wrapper(mut source: impl std::io::Read) -> std::io::Result<String> {
    let mut buffer = String::new();
    source.read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub fn data_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    let app_name = "tarium-dev";
    #[cfg(not(debug_assertions))]
    let app_name = "tarium";

    BaseDirs::new().unwrap().data_dir().join(app_name)
}

pub fn cache_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    let app_name = "tarium-dev";
    #[cfg(not(debug_assertions))]
    let app_name = "tarium";

    BaseDirs::new().unwrap().cache_dir().join(app_name)
}

pub fn config_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    let app_name = "tarium-dev";
    #[cfg(not(debug_assertions))]
    let app_name = "tarium";

    ProjectDirs::from("", "", app_name)
        .unwrap()
        .config_dir()
        .to_path_buf()
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

pub fn logs_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    let app_name = "tarium-dev";
    #[cfg(not(debug_assertions))]
    let app_name = "tarium";

    BaseDirs::new()
        .unwrap()
        .data_dir()
        .join(app_name)
        .join("logs")
}

pub fn extract_versions(s: &str) -> Vec<String> {
    VERSION_RE
        .captures_iter(s)
        .filter_map(|cap| {
            let major = cap.name("major")?.as_str();
            let minor = cap.name("minor").map(|m| m.as_str()).unwrap_or("x");
            let patch = cap.name("patch").map(|p| p.as_str()).unwrap_or("x");
            Some(format!("{}.{}.{}", major, minor, patch))
        })
        .collect()
}

pub fn is_spt_version(version: &str) -> bool {
    let known_versions = ["3.11", "3.10", "3.9"];
    for pat in known_versions {
        if version.starts_with(pat) {
            return true;
        }
    }
    false
}
