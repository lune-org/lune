use std::env::current_dir;

use anyhow::{bail, Context, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct GithubReleaseAsset {
    id: u64,
    url: String,
    name: Option<String>,
    label: Option<String>,
    content_type: String,
    size: u64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GithubRelease {
    id: u64,
    url: String,
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    draft: bool,
    prerelease: bool,
    assets: Vec<GithubReleaseAsset>,
}

pub struct GithubClient {
    client: Client,
    github_owner: String,
    github_repo: String,
}

impl GithubClient {
    pub fn new() -> Result<Self> {
        let (github_owner, github_repo) = env!("CARGO_PKG_REPOSITORY")
            .strip_prefix("https://github.com/")
            .unwrap()
            .split_once('/')
            .unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_str(&format!("{}-{}-cli", github_owner, github_repo))?,
        );
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self {
            client,
            github_owner: github_owner.to_string(),
            github_repo: github_repo.to_string(),
        })
    }

    pub async fn fetch_releases(&self) -> Result<Vec<GithubRelease>> {
        let release_api_url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            &self.github_owner, &self.github_repo
        );
        let response_bytes = self
            .client
            .get(release_api_url)
            .send()
            .await
            .context("Failed to send releases request")?
            .bytes()
            .await
            .context("Failed to get releases response bytes")?;
        let response_body: Vec<GithubRelease> = serde_json::from_slice(&response_bytes)?;
        Ok(response_body)
    }

    pub async fn fetch_release_for_this_version(&self) -> Result<GithubRelease> {
        let release_version_tag = format!("v{}", env!("CARGO_PKG_VERSION"));
        let all_releases = self.fetch_releases().await?;
        all_releases
            .iter()
            .find(|release| release.tag_name == release_version_tag)
            .map(|release| release.to_owned())
            .with_context(|| format!("Failed to find release for version {}", release_version_tag))
    }

    pub async fn fetch_release_asset(
        &self,
        release: &GithubRelease,
        asset_name: &str,
    ) -> Result<()> {
        if let Some(asset) = release
            .assets
            .iter()
            .find(|asset| matches!(&asset.name, Some(name) if name == asset_name))
        {
            let file_path = current_dir()?.join(asset_name);
            let file_bytes = self
                .client
                .get(&asset.url)
                .header("Accept", "application/octet-stream")
                .send()
                .await
                .context("Failed to send asset download request")?
                .bytes()
                .await
                .context("Failed to get asset download response bytes")?;
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

pub fn pretty_print_luau_error(e: &mlua::Error) {
    match e {
        mlua::Error::RuntimeError(e) => {
            eprintln!("{}", e);
        }
        mlua::Error::CallbackError { cause, traceback } => {
            pretty_print_luau_error(cause.as_ref());
            eprintln!("Traceback:");
            eprintln!("{}", traceback.strip_prefix("stack traceback:\n").unwrap());
        }
        mlua::Error::ToLuaConversionError { from, to, message } => {
            let msg = message
                .clone()
                .map(|m| format!("\nDetails:\n\t{m}"))
                .unwrap_or_else(|| "".to_string());
            eprintln!(
                "Failed to convert Rust type '{}' into Luau type '{}'!{}",
                from, to, msg
            )
        }
        mlua::Error::FromLuaConversionError { from, to, message } => {
            let msg = message
                .clone()
                .map(|m| format!("\nDetails:\n\t{m}"))
                .unwrap_or_else(|| "".to_string());
            eprintln!(
                "Failed to convert Luau type '{}' into Rust type '{}'!{}",
                from, to, msg
            )
        }
        e => eprintln!("{e}"),
    }
}
