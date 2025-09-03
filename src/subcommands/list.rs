use crate::TICK;
use anyhow::{Context as _, Result};
use colored::Colorize as _;
use libarov::{
    config::structs::{ModIdentifier, Profile},
    iter_ext::IterExt as _,
    GITHUB_API,
};
use octocrab::models::{repos::Release, Repository};
use tokio::task::JoinSet;

enum Metadata {
    GH(Box<Repository>, Vec<Release>),
}
impl Metadata {
    fn name(&self) -> &str {
        match self {
            Metadata::GH(p, _) => &p.name,
        }
    }

    #[expect(clippy::unwrap_used)]
    fn id(&self) -> ModIdentifier {
        match self {
            Metadata::GH(p, _) => {
                ModIdentifier::GitHubRepository(p.owner.clone().unwrap().login, p.name.clone())
            }
        }
    }

    fn slug(&self) -> &str {
        match self {
            Metadata::GH(p, _) => &p.name,
        }
    }
}

pub async fn verbose(profile: &mut Profile, markdown: bool) -> Result<()> {
    if !markdown {
        eprint!("Querying metadata... ");
    }

    let mut tasks = JoinSet::new();
    for mod_ in &profile.mods {
        match mod_.identifier.clone() {
            ModIdentifier::GitHubRepository(owner, repo) => {
                let repo = GITHUB_API.repos(owner, repo);
                tasks.spawn(async move {
                    Ok::<_, anyhow::Error>((
                        repo.get().await?,
                        repo.releases().list().send().await?,
                    ))
                });
            }
            _ => todo!(),
        }
    }

    let mut metadata = Vec::new();
    for res in tasks.join_all().await {
        let (repo, releases) = res?;
        metadata.push(Metadata::GH(Box::new(repo), releases.items));
    }
    metadata.sort_unstable_by_key(|e| e.name().to_lowercase());

    if !markdown {
        println!("{}", &*TICK);
    }

    for project in &metadata {
        let mod_ = profile
            .mods
            .iter_mut()
            .find(|mod_| mod_.identifier == project.id())
            .context("Could not find expected mod")?;

        mod_.name = project.name().to_string();
        mod_.slug = Some(project.slug().to_string());

        if markdown {
            match project {
                Metadata::GH(p, _) => github_md(p),
            }
        } else {
            match project {
                Metadata::GH(p, r) => github(p, r),
            }
        }
    }

    Ok(())
}

#[expect(clippy::unwrap_used)]
pub fn github(repo: &Repository, releases: &[Release]) {
    // Calculate number of downloads
    let mut downloads = 0;
    for release in releases {
        for asset in &release.assets {
            downloads += asset.download_count;
        }
    }

    println!(
        "
{}{}\n
  Link:         {}
  Source:       {}
  Identifier:   {}
  Open Source:  {}
  Downloads:    {}
  Authors:      {}
  Topics:       {}
  License:      {}",
        &repo.name.bold(),
        repo.description
            .as_ref()
            .map_or(String::new(), |description| {
                format!("\n  {description}")
            })
            .italic(),
        repo.html_url
            .as_ref()
            .unwrap()
            .to_string()
            .blue()
            .underline(),
        "GitHub Repository".dimmed(),
        repo.full_name.as_ref().unwrap().dimmed(),
        "Yes".green(),
        downloads.to_string().yellow(),
        repo.owner.as_ref().unwrap().login.cyan(),
        repo.topics.as_ref().map_or("".into(), |topics| topics
            .iter()
            .display(", ")
            .to_string()
            .magenta()),
        repo.license
            .as_ref()
            .map_or("None".into(), |license| format!(
                "{}{}",
                license.name,
                license.html_url.as_ref().map_or(String::new(), |url| {
                    format!(" ({})", url.to_string().blue().underline())
                })
            )),
    );
}

#[expect(clippy::unwrap_used)]
pub fn github_md(repo: &Repository) {
    println!(
        "
**[{}]({})**{}

|             |             |
|-------------|-------------|
| Source      | GitHub `{}` |
| Open Source | Yes         |
| Owner       | [{}]({})    |{}",
        repo.name,
        repo.html_url.as_ref().unwrap(),
        repo.description
            .as_ref()
            .map_or(String::new(), |description| {
                format!("  \n_{}_", description.trim())
            }),
        repo.full_name.as_ref().unwrap(),
        repo.owner.as_ref().unwrap().login,
        repo.owner.as_ref().unwrap().html_url,
        repo.topics.as_ref().map_or(String::new(), |topics| format!(
            "\n| Topics | {} |",
            topics.iter().display(", ")
        )),
    );
}
