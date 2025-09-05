use crate::{
    config::{
        filters::{Filter, ReleaseChannel},
        structs::{ModIdentifier, Profile},
    },
    extract_versions, is_spt_version,
    iter_ext::IterExt as _,
    upgrade::{check, Metadata},
    GITHUB_API,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(
        "The developer of this project has denied third party applications from downloading it"
    )]
    /// The user can manually download the mod and place it in the `user` folder of the output directory to mitigate this.
    /// However, they will have to manually update the mod.
    DistributionDenied,
    #[error("The project has already been added")]
    AlreadyAdded,
    #[error("The project is not compatible because {_0}")]
    Incompatible(#[from] check::Error),
    #[error("The project does not exist")]
    DoesNotExist,
    #[error("The project is not a mod")]
    NotAMod,
    #[error("GitHub: {0}")]
    GitHubError(String),
    #[error("GitHub: {0:#?}")]
    OctocrabError(#[from] octocrab::Error),
}
type Result<T> = std::result::Result<T, Error>;

pub fn parse_id(id: String) -> ModIdentifier {
    let split = id.split('/').collect_vec();
    ModIdentifier::GitHubRepository(split[0].to_owned(), split[1].to_owned())
}

/// Adds mods from `identifiers`, and returns successful mods with their names, and unsuccessful mods with an error.
/// Currently does not batch requests when adding multiple pinned mods.
///
/// Classifies the `identifiers` into the appropriate platforms, sends batch requests to get the necessary information,
/// checks details about the projects, and adds them to `profile` if suitable.
/// Performs checks on the mods to see whether they're compatible with the profile if `perform_checks` is true
pub async fn add(
    profile: &mut Profile,
    identifiers: Vec<ModIdentifier>,
    perform_checks: bool,
    override_profile: bool,
    filters: Vec<Filter>,
) -> Result<(Vec<String>, Vec<(String, Error)>)> {
    dbg!("adding", &identifiers);
    let mut gh_ids = Vec::new();
    let mut errors = Vec::new();

    for id in identifiers {
        match id {
            ModIdentifier::GitHubRepository(o, r) => gh_ids.push((o, r)),
            ModIdentifier::PinnedGitHubRepository((owner, repo), asset_id) => todo!(),
        }
    }

    let gh_repos = {
        let mut repos_data = Vec::new();

        // Process each repository using REST API instead of GraphQL
        for (owner, name) in &gh_ids {
            match fetch_repo_releases_rest(owner, name).await {
                Ok(metadata) => {
                    repos_data.push(((owner.clone(), name.clone()), metadata));
                }
                Err(err) => {
                    errors.push((format!("{}/{}", owner, name), err));
                }
            }
        }

        repos_data
    };

    let mut success_names = Vec::new();

    for (repo, asset_names) in gh_repos {
        match github(
            &repo,
            profile,
            Some(asset_names),
            override_profile,
            filters.clone(),
        )
        .await
        {
            Ok(_) => success_names.push(format!("{}/{}", repo.0, repo.1)),
            Err(err) => errors.push((format!("{}/{}", repo.0, repo.1), err)),
        }
    }

    Ok((success_names, errors))
}

/// Check if the repo of `repo_handler` exists, releases mods, and is compatible with `profile`.
/// If so, add it to the `profile`.
///
/// Returns the name of the repository to display to the user
pub async fn github(
    id: &(impl AsRef<str> + ToString, impl AsRef<str> + ToString),
    profile: &mut Profile,
    need_checks: Option<Metadata>,
    override_profile: bool,
    filters: Vec<Filter>,
) -> Result<()> {
    // Check if project has already been added
    if profile.mods.iter().any(|mod_| {
        mod_.name.eq_ignore_ascii_case(id.1.as_ref())
            || matches!(
                &mod_.identifier,
                ModIdentifier::GitHubRepository(owner, repo) if owner == id.0.as_ref() && repo == id.1.as_ref(),
            )
    }) {
        return Err(Error::AlreadyAdded);
    }

    if let Some(download_files) = need_checks {
        // Check if the repo is compatible
        // NOAH: wtf does it even do??? i guess thats where the filters go
        check::select_latest(
            vec![download_files].iter(),
            if override_profile {
                profile.filters.clone()
            } else {
                [profile.filters.clone(), filters.clone()].concat()
            },
        )
        .await?;
    }

    // Add it to the profile
    profile.push_mod(
        id.1.as_ref().trim().to_string(),
        ModIdentifier::GitHubRepository(id.0.to_string(), id.1.to_string()),
        id.1.as_ref().trim().to_string(),
        override_profile,
        filters,
    );

    Ok(())
}

/// Fetch repository releases using REST API instead of GraphQL to avoid authentication requirement
async fn fetch_repo_releases_rest(owner: &str, repo: &str) -> Result<Metadata> {
    // Get all releases for the repository using REST API
    let page = GITHUB_API
        .repos(owner, repo)
        .releases()
        .list()
        .per_page(10) // Get more releases per page
        .send()
        .await?;

    let mut all_metadata = Vec::new();

    for release in page.items {
        // TODO: check both release name and assets name for SPT version (here: game_versions)
        // Release.name (is Some() here)
        // Release.assets.<index>(Asset).name (is NOT Some() here)
        let mut found_versions = Vec::new();
        if let Some(ref release_name) = release.name {
            found_versions.push(extract_versions(release_name.as_str()));
        }

        // Get release assets for this release
        let assets_page = GITHUB_API
            .repos(owner, repo)
            .releases()
            .assets(release.id.0)
            .per_page(10) // Get more assets per page
            .send()
            .await?;

        // Convert each asset to Metadata
        for asset in assets_page.items {
            if asset.name.ends_with(".zip") || asset.name.ends_with(".7z") {
                found_versions.push(extract_versions(asset.name.as_str()));
                let game_versions = if found_versions.is_empty() {
                    None
                } else {
                    // TODO: check if there 3.11 or 3.10 or these with patch versions in the name
                    found_versions.sort_by(|a, b| b.len().cmp(&a.len())); // longest first
                    found_versions.dedup();
                    let filtered_versions = found_versions
                        .clone()
                        .into_iter()
                        .flatten()
                        .filter(|v| is_spt_version(v))
                        .collect::<Vec<_>>();
                    if filtered_versions.is_empty() {
                        None
                    } else {
                        Some(filtered_versions)
                    }
                };
                dbg!(&game_versions);

                // DEV: temporarily cut off patch version here too
                let game_versions = game_versions
                    .unwrap_or_default()
                    .into_iter()
                    .map(|v| {
                        if let Some(pos) = v.rfind('.') {
                            v[..pos].to_string()
                        } else {
                            v
                        }
                    })
                    .collect::<Vec<_>>();
                dbg!(&game_versions);

                all_metadata.push(Metadata::new(
                    release.name.as_ref().unwrap_or(&release.tag_name).clone(),
                    release.body.as_ref().cloned().unwrap_or_default(),
                    asset.name.clone(),
                    release.created_at.as_ref().cloned().unwrap_or_default(),
                    Some(game_versions),
                ));
            }
        }
    }
    dbg!("Fetched assets", &all_metadata);

    all_metadata.sort_by(|a, b| b.release_date.cmp(&a.release_date));
    let latest_release = all_metadata.first().cloned();

    // TODO: change this...
    Ok(latest_release.or(None).ok_or(Error::DoesNotExist)?)
}
