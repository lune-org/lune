use std::{
    env::current_dir,
    fmt::Write,
    io::{self, Write as IoWrite},
};

use anyhow::{bail, Context, Result};
use mlua::{MultiValue, Value};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};

const MAX_FORMAT_DEPTH: usize = 4;

const INDENT: &str = "    ";

const COLOR_RESET: &str = "\x1B[0m";
const COLOR_BLACK: &str = "\x1B[30m";
const COLOR_RED: &str = "\x1B[31m";
const COLOR_GREEN: &str = "\x1B[32m";
const COLOR_YELLOW: &str = "\x1B[33m";
const COLOR_BLUE: &str = "\x1B[34m";
const COLOR_PURPLE: &str = "\x1B[35m";
const COLOR_CYAN: &str = "\x1B[36m";
const COLOR_WHITE: &str = "\x1B[37m";

const STYLE_RESET: &str = "\x1B[22m";
const STYLE_BOLD: &str = "\x1B[1m";
const STYLE_DIM: &str = "\x1B[2m";

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
        let (github_owner, github_repo) = get_github_owner_and_repo();
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_str(&get_github_user_agent_header())?,
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
            github_owner,
            github_repo,
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
            .map(ToOwned::to_owned)
            .with_context(|| format!("Failed to find release for version {release_version_tag}"))
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

pub fn get_github_owner_and_repo() -> (String, String) {
    let (github_owner, github_repo) = env!("CARGO_PKG_REPOSITORY")
        .strip_prefix("https://github.com/")
        .unwrap()
        .split_once('/')
        .unwrap();
    (github_owner.to_owned(), github_repo.to_owned())
}

pub fn get_github_user_agent_header() -> String {
    let (github_owner, github_repo) = get_github_owner_and_repo();
    format!("{github_owner}-{github_repo}-cli")
}

// TODO: Separate utils out into github & formatting

pub fn flush_stdout() -> mlua::Result<()> {
    io::stdout().flush().map_err(mlua::Error::external)
}

pub fn print_label<S: AsRef<str>>(s: S) -> mlua::Result<()> {
    print!(
        "{}[{}{}{}{}]{} ",
        STYLE_BOLD,
        match s.as_ref().to_ascii_lowercase().as_str() {
            "info" => COLOR_BLUE,
            "warn" => COLOR_YELLOW,
            "error" => COLOR_RED,
            _ => COLOR_WHITE,
        },
        s.as_ref().to_ascii_uppercase(),
        COLOR_RESET,
        STYLE_BOLD,
        STYLE_RESET
    );
    flush_stdout()?;
    Ok(())
}

pub fn print_style<S: AsRef<str>>(s: S) -> mlua::Result<()> {
    print!(
        "{}",
        match s.as_ref() {
            "reset" => STYLE_RESET,
            "bold" => STYLE_BOLD,
            "dim" => STYLE_DIM,
            _ => {
                return Err(mlua::Error::RuntimeError(format!(
                    "The style '{}' is not a valid style name",
                    s.as_ref()
                )));
            }
        }
    );
    flush_stdout()?;
    Ok(())
}

pub fn print_color<S: AsRef<str>>(s: S) -> mlua::Result<()> {
    print!(
        "{}",
        match s.as_ref() {
            "reset" => COLOR_RESET,
            "black" => COLOR_BLACK,
            "red" => COLOR_RED,
            "green" => COLOR_GREEN,
            "yellow" => COLOR_YELLOW,
            "blue" => COLOR_BLUE,
            "purple" => COLOR_PURPLE,
            "cyan" => COLOR_CYAN,
            "white" => COLOR_WHITE,
            _ => {
                return Err(mlua::Error::RuntimeError(format!(
                    "The color '{}' is not a valid color name",
                    s.as_ref()
                )));
            }
        }
    );
    flush_stdout()?;
    Ok(())
}

fn can_be_plain_lua_table_key(s: &mlua::String) -> bool {
    let str = s.to_string_lossy().to_string();
    let first_char = str.chars().next().unwrap();
    if first_char.is_alphabetic() {
        str.chars().all(|c| c == '_' || c.is_alphanumeric())
    } else {
        false
    }
}

fn pretty_format_value(buffer: &mut String, value: &Value, depth: usize) -> anyhow::Result<()> {
    // TODO: Handle tables with cyclic references
    // TODO: Handle other types like function, userdata, ...
    match &value {
        Value::Nil => write!(buffer, "nil")?,
        Value::Boolean(true) => write!(buffer, "{COLOR_YELLOW}true{COLOR_RESET}")?,
        Value::Boolean(false) => write!(buffer, "{COLOR_YELLOW}false{COLOR_RESET}")?,
        Value::Number(n) => write!(buffer, "{COLOR_BLUE}{n}{COLOR_RESET}")?,
        Value::Integer(i) => write!(buffer, "{COLOR_BLUE}{i}{COLOR_RESET}")?,
        Value::String(s) => write!(
            buffer,
            "{}\"{}\"{}",
            COLOR_GREEN,
            s.to_string_lossy()
                .replace('"', r#"\""#)
                .replace('\n', r#"\n"#),
            COLOR_RESET
        )?,
        Value::Table(ref tab) => {
            if depth >= MAX_FORMAT_DEPTH {
                write!(buffer, "{STYLE_DIM}{{ ... }}{STYLE_RESET}")?;
            } else {
                let depth_indent = INDENT.repeat(depth);
                write!(buffer, "{STYLE_DIM}{{{STYLE_RESET}")?;
                for pair in tab.clone().pairs::<Value, Value>() {
                    let (key, value) = pair?;
                    match &key {
                        Value::String(s) if can_be_plain_lua_table_key(s) => write!(
                            buffer,
                            "\n{}{}{} {}={} ",
                            depth_indent,
                            INDENT,
                            s.to_string_lossy(),
                            STYLE_DIM,
                            STYLE_RESET
                        )?,
                        _ => {
                            write!(buffer, "\n{depth_indent}{INDENT}[")?;
                            pretty_format_value(buffer, &key, depth)?;
                            write!(buffer, "] {STYLE_DIM}={STYLE_RESET} ")?;
                        }
                    }
                    pretty_format_value(buffer, &value, depth + 1)?;
                    write!(buffer, "{STYLE_DIM},{STYLE_RESET}")?;
                }
                write!(buffer, "\n{depth_indent}{STYLE_DIM}}}{STYLE_RESET}")?;
            }
        }
        _ => write!(buffer, "?")?,
    }
    Ok(())
}

pub fn pretty_format_multi_value(multi: &MultiValue) -> mlua::Result<String> {
    let mut buffer = String::new();
    let mut counter = 0;
    for value in multi {
        counter += 1;
        if let Value::String(s) = value {
            write!(buffer, "{}", s.to_string_lossy()).map_err(mlua::Error::external)?;
        } else {
            pretty_format_value(&mut buffer, value, 0).map_err(mlua::Error::external)?;
        }
        if counter < multi.len() {
            write!(&mut buffer, " ").map_err(mlua::Error::external)?;
        }
    }
    Ok(buffer)
}

pub fn pretty_print_luau_error(e: &mlua::Error) {
    match e {
        mlua::Error::RuntimeError(e) => {
            eprintln!("{e}");
        }
        mlua::Error::CallbackError { cause, traceback } => {
            pretty_print_luau_error(cause.as_ref());
            eprintln!("Traceback:");
            eprintln!("{}", traceback.strip_prefix("stack traceback:\n").unwrap());
        }
        mlua::Error::ToLuaConversionError { from, to, message } => {
            let msg = message
                .clone()
                .map_or_else(String::new, |m| format!("\nDetails:\n\t{m}"));
            eprintln!(
                "Failed to convert Rust type '{}' into Luau type '{}'!{}",
                from, to, msg
            );
        }
        mlua::Error::FromLuaConversionError { from, to, message } => {
            let msg = message
                .clone()
                .map_or_else(String::new, |m| format!("\nDetails:\n\t{m}"));
            eprintln!(
                "Failed to convert Luau type '{}' into Rust type '{}'!{}",
                from, to, msg
            );
        }
        e => eprintln!("{e}"),
    }
}
