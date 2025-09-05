use super::{from_gh_asset, from_gh_releases, DistributionDeniedError, DownloadData};
use crate::{
    config::{
        filters::Filter,
        structs::{Mod, ModIdentifier},
    },
    GITHUB_API,
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    DistributionDenied(#[from] DistributionDeniedError),
    CheckError(#[from] super::check::Error),
    #[error("The pin provided is an invalid identifier")]
    InvalidPinID(#[from] std::num::ParseIntError),
    #[error("GitHub: {0:#?}")]
    GitHubError(#[from] octocrab::Error),
}
type Result<T> = std::result::Result<T, Error>;

impl Mod {
    pub async fn fetch_download_file(
        &self,
        mut profile_filters: Vec<Filter>,
    ) -> Result<DownloadData> {
        match &self.identifier {
            ModIdentifier::PinnedGitHubRepository((owner, repo), pin) => Ok(from_gh_asset(
                GITHUB_API
                    .repos(owner, repo)
                    .release_assets()
                    .get(*pin as u64)
                    .await?,
            )),
            id => {
                let download_files = match &id {
                    ModIdentifier::GitHubRepository(owner, repo) => GITHUB_API
                        .repos(owner, repo)
                        .releases()
                        .list()
                        .send()
                        .await
                        .map(|r| from_gh_releases(r.items))?,
                    _ => unreachable!(),
                };

                // Find the best candidate using filters
                let mut best_candidate = None;
                let filters = if self.override_filters {
                    self.filters.clone()
                } else {
                    profile_filters.extend(self.filters.clone());
                    profile_filters
                };

                // Check each candidate against all filters
                for (metadata, download_data) in &download_files {
                    let mut passes_all = true;
                    for filter in &filters {
                        if !filter.matches(metadata).await? {
                            passes_all = false;
                            break;
                        }
                    }
                    if passes_all {
                        best_candidate = Some(download_data);
                        break; // Take the first (best) match since they're sorted by preference
                    }
                }

                match best_candidate {
                    Some(download_data) => Ok(download_data.clone()),
                    None => {
                        // Fallback: if every candidate has empty game_versions and the only
                        // failing filters are GameVersion filters, allow the newest asset.
                        let all_empty_versions = download_files
                            .iter()
                            .all(|(m, _)| m.game_versions.is_empty());
                        let has_game_version_filters = filters.iter().any(|f| matches!(f, Filter::GameVersionStrict(_) | Filter::GameVersionMinor(_)));
                        if all_empty_versions && has_game_version_filters {
                            if let Some((metadata, dd)) = download_files.first() {
                                println!("Warning: no version tags found in release '{}'; using latest asset without version filtering.", metadata.title);
                                Ok(dd.clone())
                            } else {
                                Err(super::check::Error::NoCompatibleFiles.into())
                            }
                        } else {
                            Err(super::check::Error::NoCompatibleFiles.into())
                        }
                    }
                }
            }
        }
    }
}
