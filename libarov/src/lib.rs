#![cfg_attr(debug_assertions, allow(warnings))]

pub mod add;
pub mod config;
pub mod iter_ext;
pub mod upgrade;

pub use add::add;

use directories::{BaseDirs, ProjectDirs};
use std::{path::PathBuf, sync::LazyLock};

use regex::Regex;

pub static GITHUB_API: LazyLock<octocrab::Octocrab> = LazyLock::new(|| {
    let mut github = octocrab::OctocrabBuilder::new();
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        github = github.personal_token(token);
        println!("Using GitHub token for authentication");
    } else {
        println!("No GitHub token found, using unauthenticated requests");
    }
    // Set a proper User-Agent header as required by GitHub API
    github = github.add_header(
        "User-Agent".try_into().expect("Valid header name"),
        "tarium/5.0.0 (https://github.com/NQMVD/tarium)".to_string(),
    );
    println!("Set User-Agent: tarium/5.0.0 (https://github.com/NQMVD/tarium)");
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

pub static BASE_DIRS: LazyLock<BaseDirs> =
    LazyLock::new(|| BaseDirs::new().expect("Could not get OS specific directories"));

pub static PROJECT_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("", "", "tarium").expect("Could not get OS specific directories")
});

/// Gets the default Minecraft instance directory based on the current compilation `target_os`
pub fn get_spt_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        BASE_DIRS.data_dir().join("minecraft")
    }
    #[cfg(target_os = "windows")]
    {
        BASE_DIRS.data_dir().join(".minecraft")
        // TODO: get current dir and check if its name is SPT
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        BASE_DIRS.home_dir().join(".minecraft")
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
