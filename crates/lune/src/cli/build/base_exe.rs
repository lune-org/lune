use std::{
    io::{Cursor, Read},
    path::PathBuf,
};

use async_fs as fs;
use blocking::unblock;

use crate::standalone::metadata::CURRENT_EXE;

use super::{
    files::write_executable_file_to,
    result::{BuildError, BuildResult},
    target::{BuildTarget, CACHE_DIR},
};

const RELEASE_REQUEST_HEADERS: &[(&str, &str)] = &[
    (
        "User-Agent",
        concat!(
            "Lune/",
            env!("CARGO_PKG_VERSION"),
            " (",
            env!("CARGO_PKG_REPOSITORY"),
            ")"
        ),
    ),
    ("Accept", "application/octet-stream"),
    // ("Accept-Encoding", "gzip"),
];

/**
    Discovers the path to the base executable to use for cross-compilation.

    If the target is the same as the current system, the current executable is used.

    If no binary exists at the target path, it will attempt to download it from the internet.
*/
pub async fn get_or_download_base_executable(target: BuildTarget) -> BuildResult<PathBuf> {
    if target.is_current_system() {
        return Ok(CURRENT_EXE.to_path_buf());
    }
    if target.cache_path().exists() {
        return Ok(target.cache_path());
    }

    // The target is not cached, we must download it
    println!("Requested target '{target}' does not exist in cache");
    let version = env!("CARGO_PKG_VERSION");
    let target_triple = format!("lune-{version}-{target}");

    let release_url = format!(
        "{base_url}/v{version}/{target_triple}.zip",
        base_url = "https://github.com/lune-org/lune/releases/download",
    );

    // NOTE: This is not entirely accurate, but it is clearer for a user
    println!("Downloading {target_triple}{}...", target.exe_suffix());

    // Try to request to download the zip file from the target url,
    // making sure transient errors are handled gracefully and
    // with a different error message than "not found"
    let url = release_url.parse().expect("release url is valid");
    let headers = RELEASE_REQUEST_HEADERS
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();
    let res = lune_std_net::fetch(url, None, Some(headers), None)
        .await
        .map_err(BuildError::Download)?;
    let (parts, body) = res.into_inner().into_parts();

    if !parts.status.is_success() {
        if parts.status.as_u16() == 404 {
            return Err(BuildError::ReleaseTargetNotFound(target));
        }
        let body = body.into_bytes();
        return Err(BuildError::Download(format!(
            "Request was not successful\
            \nStatus: {}\
            \nBody: {}",
            parts.status,
            if body.len() > 128 {
                String::from_utf8_lossy(&body[0..128])
            } else {
                String::from_utf8_lossy(&body)
            }
        )));
    }

    // Start reading the zip file
    let zip_file = Cursor::new(body.into_bytes());

    // Look for and extract the binary file from the zip file
    // NOTE: We use spawn_blocking here since reading a zip
    // archive is a somewhat slow / blocking operation
    let binary_file_name = format!("lune{}", target.exe_suffix());
    let binary_file_handle = unblock(move || {
        let mut archive = zip::ZipArchive::new(zip_file)?;

        let mut binary = Vec::new();
        archive
            .by_name(&binary_file_name)
            .or(Err(BuildError::ZippedBinaryNotFound(binary_file_name)))?
            .read_to_end(&mut binary)?;

        Ok::<_, BuildError>(binary)
    });
    let binary_file_contents = binary_file_handle.await?;

    // Finally, write the extracted binary to the cache
    if !CACHE_DIR.exists() {
        fs::create_dir_all(CACHE_DIR.as_path()).await?;
    }
    write_executable_file_to(target.cache_path(), binary_file_contents).await?;
    println!("Downloaded successfully and added to cache");

    Ok(target.cache_path())
}
