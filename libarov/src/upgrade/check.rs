use super::Metadata;
use crate::{
    config::filters::{Filter, ReleaseChannel},
    iter_ext::IterExt,
};
use log::{debug, info, warn};
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
        debug!(SCOPE = "libarov::upgrade::check", groups_len = v.len(); "version groups cache hit");
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

        let g = VERSION_GROUPS.get().unwrap();
        info!(SCOPE = "libarov::upgrade::check", groups_len = g.len(); "version groups initialised");

        Ok(VERSION_GROUPS.get().unwrap())
    }
}

impl Filter {
    /// Returns whether the given metadata passes through this filter
    pub async fn matches(&self, metadata: &Metadata) -> Result<bool> {
        let result = match self {
            Filter::GameVersionStrict(versions) => {
                debug!(SCOPE = "libarov::upgrade::check", filter:display = self, filename = metadata.filename.as_str(); "checking filter");
                versions.iter().any(|v| {
                    if metadata.game_versions.contains(v) {
                        return true;
                    }
                    // Accept minor-only vs minor.x equivalence
                    if v.ends_with(".x") {
                        let trimmed = &v[..v.len() - 2];
                        if metadata.game_versions.contains(&trimmed.to_string()) {
                            return true;
                        }
                    } else if v.chars().filter(|c| *c == '.').count() == 1 {
                        // major.minor form
                        let with_x = format!("{v}.x");
                        if metadata.game_versions.contains(&with_x) {
                            return true;
                        }
                    }
                    false
                })
            }

            Filter::GameVersionMinor(versions) => {
                debug!(
                    SCOPE = "libarov::upgrade::check",
                    filter:display = self,
                    filename = metadata.filename.as_str(),
                    requested:debug = versions,
                    metadata_versions:debug = &metadata.game_versions;
                    "checking filter"
                );
                let mut final_versions = vec![];
                for group in get_version_groups().await? {
                    let hit = group.iter().any(|v| versions.contains(v));
                    debug!(
                        SCOPE = "libarov::upgrade::check",
                        group_len = group.len(),
                        hit = hit;
                        "considering version group"
                    );
                    if hit {
                        final_versions.extend(group.clone());
                    }
                }
                debug!(
                    SCOPE = "libarov::upgrade::check",
                    final_versions_len = final_versions.len(),
                    final_versions:debug = &final_versions;
                    "expanded minor-compatible versions"
                );

                let mut matched = false;
                let mut matched_by: &str = "";
                let mut matched_value: String = String::new();
                for v in &final_versions {
                    if metadata.game_versions.contains(v) {
                        matched = true;
                        matched_by = "exact";
                        matched_value = v.clone();
                        break;
                    }
                    if v.ends_with(".x") {
                        let trimmed = &v[..v.len() - 2];
                        if metadata.game_versions.contains(&trimmed.to_string()) {
                            matched = true;
                            matched_by = "wildcard->exact";
                            matched_value = v.clone();
                            break;
                        }
                    } else if v.chars().filter(|c| *c == '.').count() == 1 {
                        let with_x = format!("{v}.x");
                        if metadata.game_versions.contains(&with_x) {
                            matched = true;
                            matched_by = "minor->wildcard";
                            matched_value = with_x;
                            break;
                        }
                    }
                }

                if matched {
                    debug!(
                        SCOPE = "libarov::upgrade::check",
                        matched_by,
                        matched_value:debug = matched_value,
                        metadata_versions:debug = &metadata.game_versions;
                        "minor filter matched"
                    );
                } else {
                    debug!(
                        SCOPE = "libarov::upgrade::check",
                        final_versions_len = final_versions.len(),
                        metadata_versions:debug = &metadata.game_versions;
                        "minor filter no match"
                    );
                }
                matched
            }

            Filter::ReleaseChannel(channel) => match channel {
                ReleaseChannel::Alpha => true,
                ReleaseChannel::Beta => {
                    metadata.channel == ReleaseChannel::Beta
                        || metadata.channel == ReleaseChannel::Release
                }
                ReleaseChannel::Release => metadata.channel == ReleaseChannel::Release,
            },

            Filter::Filename(regex) => {
                debug!(SCOPE = "libarov::upgrade::check", pattern = regex.as_str(), filename = metadata.filename.as_str(); "compiling filename regex");
                let regex = Regex::new(regex)?;
                regex.is_match(&metadata.filename)
            }

            Filter::Title(regex) => {
                debug!(SCOPE = "libarov::upgrade::check", pattern = regex.as_str(), title = metadata.title.as_str(); "compiling title regex");
                let regex = Regex::new(regex)?;
                regex.is_match(&metadata.title)
            }

            Filter::Description(regex) => {
                debug!(SCOPE = "libarov::upgrade::check", pattern = regex.as_str(); "compiling description regex");
                let regex = Regex::new(regex)?;
                regex.is_match(&metadata.description)
            }
        };

        info!(
            SCOPE = "libarov::upgrade::check",
            filter:display = self,
            title = metadata.title.as_str(),
            filename = metadata.filename.as_str(),
            passed = result;
            "filter evaluated"
        );
        Ok(result)
    }

    /// Filters an iterator of metadata, returning only items that pass the filter
    pub async fn filter<'a>(
        &self,
        metadata_iter: impl Iterator<Item = &'a Metadata> + 'a,
    ) -> Result<impl Iterator<Item = &'a Metadata> + 'a> {
        // For now, collect into a Vec since we need to handle async matching
        // A more sophisticated approach would use a custom iterator
        let mut results = Vec::new();
        let mut total = 0usize;
        for metadata in metadata_iter {
            total += 1;
            if self.matches(metadata).await? {
                results.push(metadata);
            }
        }
        info!(SCOPE = "libarov::upgrade::check", filter:display = self, total = total, matched = results.len(); "filtering complete");
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
    info!(SCOPE = "libarov::upgrade::check", candidates = candidates_vec.len(), filters:debug = filters; "select_latest invoked");

    if candidates_vec.is_empty() {
        warn!(SCOPE = "libarov::upgrade::check"; "no candidates provided");
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
        warn!(SCOPE = "libarov::upgrade::check", empty_filters:debug = empty_filters; "one or more filters matched nothing");
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
            info!(SCOPE = "libarov::upgrade::check", filename = candidate.filename.as_str(), title = candidate.title.as_str(); "selected candidate");
            return Ok(candidate);
        }
    }

    warn!(SCOPE = "libarov::upgrade::check"; "no compatible files after applying filters");
    Err(Error::NoCompatibleFiles)
}
