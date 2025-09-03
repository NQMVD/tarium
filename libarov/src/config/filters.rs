use crate::iter_ext::IterExt as _;
use derive_more::derive::Display;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Display, Clone)]
pub enum Filter {
    /// Selects files strictly compatible with the versions specified
    #[display("Game Version ({})", _0.iter().display(", "))]
    GameVersionStrict(Vec<String>),

    /// Selects files compatible with the versions specified and related versions that are
    /// considered to not have breaking changes (determined using Modrinth's game version tag list)
    #[display("Game Version Minor ({})", _0.iter().display(", "))]
    GameVersionMinor(Vec<String>),

    /// Selects files matching the channel provided or more stable channels
    #[display("Release Channel ({_0})")]
    ReleaseChannel(ReleaseChannel),

    /// Selects the files with filenames matching the provided regex
    #[display("Filename ({_0})")]
    Filename(String),

    /// Selects files with titles matching the provided regex
    #[display("Title ({_0})")]
    Title(String),

    /// Selects files with descriptions matching the provided regex
    #[display("Description ({_0})")]
    Description(String),
}

pub trait ProfileParameters {
    /// Get the game versions present, if self has `GameVersionStrict` or `GameVersionMinor`
    fn game_versions(&self) -> Option<&Vec<String>>;
    /// Get the game versions present, if self has `GameVersionStrict` or `GameVersionMinor`
    fn game_versions_mut(&mut self) -> Option<&mut Vec<String>>;
}

impl ProfileParameters for Vec<Filter> {
    fn game_versions(&self) -> Option<&Vec<String>> {
        self.iter().find_map(|filter| match filter {
            Filter::GameVersionStrict(v) => Some(v),
            Filter::GameVersionMinor(v) => Some(v),
            _ => None,
        })
    }

    fn game_versions_mut(&mut self) -> Option<&mut Vec<String>> {
        self.iter_mut().find_map(|filter| match filter {
            Filter::GameVersionStrict(v) => Some(v),
            Filter::GameVersionMinor(v) => Some(v),
            _ => None,
        })
    }
}

// impl PartialEq for Filter {
//     fn eq(&self, other: &Self) -> bool {
//         discriminant(self) == discriminant(other)
//     }
// }

#[derive(Deserialize, Serialize, Debug, Display, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ReleaseChannel {
    Release,
    Beta,
    Alpha,
}
