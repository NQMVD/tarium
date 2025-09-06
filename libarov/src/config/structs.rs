use super::filters::Filter;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Config {
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub active_profile: usize,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub profiles: Vec<Profile>,

    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub active_modpack: usize,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub modpacks: Vec<Modpack>,
}

const fn is_zero(n: &usize) -> bool {
    *n == 0
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Modpack {
    pub name: String,
    pub output_dir: PathBuf,
    pub install_overrides: bool,
    pub identifier: ModpackIdentifier,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModpackIdentifier {
    CurseForgeModpack(i32),
    ModrinthModpack(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Profile {
    pub name: String,

    /// The directory to download mod files to
    pub output_dir: PathBuf,

    // There will be no filters when reading a v4 config
    #[serde(default)]
    pub filters: Vec<Filter>,

    pub mods: Vec<Mod>,
    // Kept for backwards compatibility reasons (i.e. migrating from a v4 config)
    // #[serde(skip_serializing)]
    // game_version: Option<String>,
}

impl Profile {
    /// A simple constructor that automatically deals with converting to filters
    pub fn new(
        name: String,
        output_dir: PathBuf,
        game_versions: Vec<String>,
        strict: bool,
    ) -> Self {
        // cutoff patch segment from supplied versions (remove patch component)
        let game_versions = game_versions
            .into_iter()
            .map(|v| {
                // cutoff patch segment by removing patch version at last dot
                if let Some(pos) = v.rfind('.') {
                    // need String here
                    v[..pos].to_string()
                } else {
                    v
                }
            })
            .collect::<Vec<_>>();

        let filters = vec![if strict {
            Filter::GameVersionStrict(game_versions)
        } else {
            Filter::GameVersionMinor(game_versions)
        }];

        Self {
            name,
            output_dir,
            filters,
            mods: vec![],
            // game_version: None,
        }
    }

    pub fn push_mod(
        &mut self,
        name: String,
        identifier: ModIdentifier,
        slug: String,
        // filters: Vec<Filter>,
    ) {
        self.mods.push(Mod {
            name,
            slug: Some(slug),
            identifier,
            // filters,
            // check_game_version: None,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub identifier: ModIdentifier,

    // Is an `Option` for backwards compatibility reasons,
    // since the slug field didn't exist in older ferium versions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    // Custom filters that apply only for this mod
    // #[serde(skip_serializing_if = "Vec::is_empty")]
    // #[serde(default)]
    // pub filters: Vec<Filter>,
    // Whether the filters specified above replace or apply with the profile's filters
    // #[serde(skip_serializing_if = "is_false")]
    // #[serde(default)]
    // pub override_filters: bool,
    // Kept for backwards compatibility reasons
    // #[serde(skip_serializing)]
    // check_game_version: Option<bool>,
}

impl Mod {
    pub fn new(name: String, identifier: ModIdentifier, filters: Vec<Filter>) -> Self {
        Self {
            name,
            slug: None,
            identifier,
            // filters,
            // check_game_version: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModIdentifier {
    GitHubRepository(String, String),

    PinnedGitHubRepository((String, String), i32),
}
