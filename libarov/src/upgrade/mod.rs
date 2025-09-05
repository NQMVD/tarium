pub mod check;
pub mod mod_downloadable;

use crate::{
    config::{filters::ReleaseChannel, structs::ModIdentifier}, extract_versions, is_spt_version, iter_ext::IterExt as _
};
use chrono::{DateTime, Utc};
use octocrab::models::repos::{Asset as GHAsset, Release as GHRelease};
use reqwest::{Client, Url};
use std::{
    fs::{create_dir_all, rename, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    ReqwestError(#[from] reqwest::Error),
    IOError(#[from] std::io::Error),
}
type Result<T> = std::result::Result<T, Error>;

/// Metadata about a GitHub Release
#[derive(Debug, Clone)]
pub struct Metadata {
    /// The title of the GitHub Release
    pub title: String,
    /// The body of the GitHub Release
    pub description: String,
    /// The filename of the asset
    pub filename: String,
    /// The release date
    pub release_date: DateTime<Utc>,
    /// The release channel (e.g. stable, beta)
    pub channel: ReleaseChannel,
    /// The game versions this release is compatible with
    pub game_versions: Vec<String>,
}

impl Metadata {
    pub fn new(
        title: String,
        description: String,
        filename: String,
        release_date: DateTime<Utc>,
        game_versions: Option<Vec<String>>, // not all releases have game versions provided
    ) -> Self {
        Metadata {
            title,
            description,
            filename,
            release_date,
            channel: ReleaseChannel::Release,
            game_versions: game_versions.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadData {
    pub download_url: Url,
    /// The path of the downloaded file relative to the output directory
    ///
    /// The filename by default, but can be configured with subdirectories for modpacks.
    pub output: PathBuf,
    /// The length of the file in bytes
    pub length: usize,
    /// The dependencies this file has
    pub dependencies: Vec<ModIdentifier>,
    /// Other mods this file is incompatible with
    pub conflicts: Vec<ModIdentifier>,
}

#[derive(Debug, thiserror::Error)]
#[error("The developer of this project has denied third party applications from downloading it")]
/// Contains the mod ID and file ID
pub struct DistributionDeniedError(pub i32, pub i32);

// pub fn from_modpack_file(file: ModpackModFile) -> DownloadData {
//     DownloadData {
//         download_url: file
//             .downloads
//             .first()
//             .expect("Download URLs not provided")
//             .clone(),
//         output: file.path,
//         length: file.file_size,
//         dependencies: Vec::new(),
//         conflicts: Vec::new(),
//     }
// }

// TODO: de-duplicate this? also in add.rs, from switch to REST calls
pub fn from_gh_releases(
    releases: impl IntoIterator<Item = GHRelease>,
) -> Vec<(Metadata, DownloadData)> {
    releases
        .into_iter()
        .flat_map(|release| {
            let mut found_versions = Vec::new();
            if let Some(ref release_name) = release.name {
                found_versions.push(extract_versions(release_name.as_str()));
            }

            release
                .assets
                .into_iter()
                // Only consider archive assets we can process
                .filter(|asset| asset.name.ends_with(".zip") || asset.name.ends_with(".7z"))
                .map(move |asset| {
                    found_versions.push(extract_versions(asset.name.as_str()));
                    let game_versions = if found_versions.is_empty() {
                        None
                    } else {
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
                            // Normalize to major.minor by trimming patch/wildcard segment
                            let normalized = filtered_versions
                                .into_iter()
                                .map(|v| {
                                    if v.matches('.').count() == 2 {
                                        // major.minor.patch or major.minor.x
                                        if let Some(pos) = v.rfind('.') { v[..pos].to_string() } else { v }
                                    } else { v }
                                })
                                .collect::<Vec<_>>();
                            Some(normalized)
                        }
                    };

                    (
                        Metadata::new(
                            release.name.clone().unwrap_or_default(),
                            release.body.clone().unwrap_or_default(),
                            asset.name.clone(),
                            release.published_at.unwrap_or_else(Utc::now),
                            game_versions,
                        ),
                        DownloadData {
                            download_url: asset.browser_download_url,
                            output: asset.name.into(),
                            length: asset.size as usize,
                            dependencies: Vec::new(),
                            conflicts: Vec::new(),
                        },
                    )
                })
        })
        .collect_vec()
}

pub fn from_gh_asset(asset: GHAsset) -> DownloadData {
    DownloadData {
        download_url: asset.browser_download_url,
        output: asset.name.into(),
        length: asset.size as usize,
        dependencies: Vec::new(),
        conflicts: Vec::new(),
    }
}

impl DownloadData {
    /// Consumes `self` and downloads the file to the `output_dir`
    ///
    /// The `update` closure is called with the chunk length whenever a chunk is downloaded and written.
    ///
    /// Returns the total size of the file and the filename.
    pub async fn download(
        self,
        client: Client,
        output_dir: impl AsRef<Path>,
        update: impl Fn(usize) + Send,
    ) -> Result<(usize, String)> {
        let (filename, url, size) = (self.filename(), self.download_url, self.length);
        let out_file_path = output_dir.as_ref().join(&self.output);
        let temp_file_path = out_file_path.with_extension("part");
        if let Some(up_dir) = out_file_path.parent() {
            create_dir_all(up_dir)?;
        }

        let mut temp_file = BufWriter::with_capacity(
            size,
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&temp_file_path)?,
        );

        let mut response = client.get(url).send().await?;

        while let Some(chunk) = response.chunk().await? {
            temp_file.write_all(&chunk)?;
            update(chunk.len());
        }
        temp_file.flush()?;
        rename(temp_file_path, &out_file_path)?;

        #[cfg(windows)]
        {
            if let Ok(meta) = std::fs::metadata(&out_file_path) {
                if meta.permissions().readonly() {
                    let mut perms = meta.permissions();
                    perms.set_readonly(false);
                    let _ = std::fs::set_permissions(&out_file_path, perms);
                }
            }
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&out_file_path) {
                let mut perms = meta.permissions();
                // rw-r--r--
                let _ = perms.set_mode(0o644);
                let _ = std::fs::set_permissions(&out_file_path, perms);
            }
        }

        Ok((size, filename))
    }

    pub fn filename(&self) -> String {
        self.output
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
}
