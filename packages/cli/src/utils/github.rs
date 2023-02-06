use std::env::current_dir;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use lune::utils::net::{get_github_owner_and_repo, get_request_user_agent_header};

#[derive(Clone, Deserialize, Serialize)]
pub struct ReleaseAsset {
    id: u64,
    url: String,
    name: Option<String>,
    label: Option<String>,
    content_type: String,
    size: u64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Release {
    id: u64,
    url: String,
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    draft: bool,
    prerelease: bool,
    assets: Vec<ReleaseAsset>,
}

pub struct Client {
    github_owner: String,
    github_repo: String,
}

impl Client {
    pub fn new() -> Self {
        let (github_owner, github_repo) = get_github_owner_and_repo();
        Self {
            github_owner,
            github_repo,
        }
    }

    async fn get(&self, url: &str, accept: Option<&str>) -> Result<Vec<u8>> {
        let request = reqwest::ClientBuilder::new()
            .build()?
            .request(reqwest::Method::GET, url)
            .header("User-Agent", &get_request_user_agent_header())
            .header("Accept", accept.unwrap_or("application/vnd.github+json"))
            .header("X-GitHub-Api-Version", "2022-11-28");
        Ok(request.send().await?.bytes().await?.to_vec())
    }

    pub async fn fetch_releases(&self) -> Result<Vec<Release>> {
        let release_api_url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            &self.github_owner, &self.github_repo
        );
        let response_bytes = self.get(&release_api_url, None).await?;
        Ok(serde_json::from_slice(&response_bytes)?)
    }

    pub async fn fetch_release_for_this_version(&self) -> Result<Release> {
        let release_version_tag = format!("v{}", env!("CARGO_PKG_VERSION"));
        let all_releases = self.fetch_releases().await?;
        all_releases
            .iter()
            .find(|release| release.tag_name == release_version_tag)
            .map(ToOwned::to_owned)
            .with_context(|| format!("Failed to find release for version {release_version_tag}"))
    }

    pub async fn fetch_release_asset(&self, release: &Release, asset_name: &str) -> Result<()> {
        if let Some(asset) = release
            .assets
            .iter()
            .find(|asset| matches!(&asset.name, Some(name) if name == asset_name))
        {
            let file_path = current_dir()?.join(asset_name);
            let file_bytes = self
                .get(&asset.url, Some("application/octet-stream"))
                .await?;
            tokio::fs::write(&file_path, &file_bytes)
                .await
                .with_context(|| {
                    format!("Failed to write file at path '{}'", &file_path.display())
                })?;
        } else {
            bail!(
                "Failed to find release asset '{}' for release '{}'",
                asset_name,
                &release.tag_name
            )
        }
        Ok(())
    }
}
