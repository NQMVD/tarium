use super::Metadata;
use crate::{
    config::filters::{Filter, ReleaseChannel},
    iter_ext::IterExt,
};
use regex::Regex;
use std::sync::OnceLock;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum Error {
    FilenameRegex(#[from] regex::Error),
    #[error("The following filter(s) were empty: {}", _0.iter().display(", "))]
    FilterEmpty(Vec<String>),
    #[error("No compatible files found after applying all filters")]
    NoCompatibleFiles,
}
pub type Result<T> = std::result::Result<T, Error>;

static VERSION_GROUPS: OnceLock<Vec<Vec<String>>> = OnceLock::new();

/// Gets groups of versions that are considered minor updates in terms of mod compatibility
///
/// This is determined by Modrinth's `major` parameter for game versions.
pub async fn get_version_groups() -> Result<&'static Vec<Vec<String>>> {
    if let Some(v) = VERSION_GROUPS.get() {
        Ok(v)
    } else {
        // TODO: port this???
        // let versions = MODRINTH_API.tag_list_game_versions().await?;
        let v = vec![vec![]];
        // for version in versions {
        //     if version.version_type == GameVersionType::Release {
        //         // Push the version to the latest group
        //         v.last_mut().unwrap().push(version.version);
        //         // Create a new group if a new major versions is present
        //         if version.major {
        //             v.push(vec![]);
        //         }
        //     }
        // }
        let _ = VERSION_GROUPS.set(v);

        Ok(VERSION_GROUPS.get().unwrap())
    }
}

impl Filter {
    /// Returns whether the given metadata passes through this filter
    pub async fn matches(&self, metadata: &Metadata) -> Result<bool> {
        Ok(match self {
            Filter::GameVersionStrict(versions) => {
                versions.iter().any(|v| {
                    if metadata.game_versions.contains(v) { return true; }
                    // Accept minor-only vs minor.x equivalence
                    if v.ends_with(".x") {
                        let trimmed = &v[..v.len()-2];
                        if metadata.game_versions.contains(&trimmed.to_string()) { return true; }
                    } else if v.chars().filter(|c| *c == '.').count() == 1 { // major.minor form
                        let with_x = format!("{v}.x");
                        if metadata.game_versions.contains(&with_x) { return true; }
                    }
                    false
                })
            }

            Filter::GameVersionMinor(versions) => {
                let mut final_versions = vec![];
                for group in get_version_groups().await? {
                    if group.iter().any(|v| versions.contains(v)) {
                        final_versions.extend(group.clone());
                    }
                }
                final_versions.iter().any(|v| {
                    if metadata.game_versions.contains(v) { return true; }
                    if v.ends_with(".x") {
                        let trimmed = &v[..v.len()-2];
                        if metadata.game_versions.contains(&trimmed.to_string()) { return true; }
                    } else if v.chars().filter(|c| *c == '.').count() == 1 {
                        let with_x = format!("{v}.x");
                        if metadata.game_versions.contains(&with_x) { return true; }
                    }
                    false
                })
            }

            Filter::ReleaseChannel(channel) => match channel {
                ReleaseChannel::Alpha => true,
                ReleaseChannel::Beta => {
                    metadata.channel == ReleaseChannel::Beta || metadata.channel == ReleaseChannel::Release
                }
                ReleaseChannel::Release => metadata.channel == ReleaseChannel::Release,
            },

            Filter::Filename(regex) => {
                let regex = Regex::new(regex)?;
                regex.is_match(&metadata.filename)
            }

            Filter::Title(regex) => {
                let regex = Regex::new(regex)?;
                regex.is_match(&metadata.title)
            }

            Filter::Description(regex) => {
                let regex = Regex::new(regex)?;
                regex.is_match(&metadata.description)
            }
        })
    }

    /// Filters an iterator of metadata, returning only items that pass the filter
    pub async fn filter<'a>(
        &self,
        metadata_iter: impl Iterator<Item = &'a Metadata> + 'a,
    ) -> Result<impl Iterator<Item = &'a Metadata> + 'a> {
        // For now, collect into a Vec since we need to handle async matching
        // A more sophisticated approach would use a custom iterator
        let mut results = Vec::new();
        for metadata in metadata_iter {
            if self.matches(metadata).await? {
                results.push(metadata);
            }
        }
        Ok(results.into_iter())
    }
}

/// Apply multiple filters and select the best matching metadata from the provided candidates.
/// The candidates should be sorted in order of preference (e.g., chronological, newest first).
/// 
/// Returns the selected metadata that passes all filters, or an error if no candidates pass.
pub async fn select_latest<'a>(
    candidates: impl Iterator<Item = &'a Metadata> + Clone,
    filters: Vec<Filter>,
) -> Result<&'a Metadata> {
    let candidates_vec: Vec<&Metadata> = candidates.collect();
    
    if candidates_vec.is_empty() {
        return Err(Error::NoCompatibleFiles);
    }

    // Check which filters produce empty results first
    let mut empty_filters = Vec::new();
    for filter in &filters {
        let mut has_match = false;
        for &candidate in &candidates_vec {
            if filter.matches(candidate).await? {
                has_match = true;
                break;
            }
        }
        if !has_match {
            empty_filters.push(filter.to_string());
        }
    }

    if !empty_filters.is_empty() {
        return Err(Error::FilterEmpty(empty_filters));
    }

    // Find the first candidate that passes all filters
    for &candidate in &candidates_vec {
        let mut passes_all = true;
        for filter in &filters {
            if !filter.matches(candidate).await? {
                passes_all = false;
                break;
            }
        }
        if passes_all {
            return Ok(candidate);
        }
    }

    Err(Error::NoCompatibleFiles)
}
