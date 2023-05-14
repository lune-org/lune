use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use directories::UserDirs;

use futures_util::future::try_join_all;
use tokio::fs::{create_dir_all, write};

#[allow(clippy::too_many_lines)]
pub async fn generate_from_type_definitions(
    typedef_files: HashMap<String, Vec<u8>>,
) -> Result<HashMap<String, PathBuf>> {
    let mut dirs_to_write = Vec::new();
    let mut files_to_write = Vec::new();
    // Create the typedefs dir in the users cache dir
    let cache_dir = UserDirs::new()
        .context("Failed to find user home directory")?
        .home_dir()
        .join(".lune")
        .join(".typedefs")
        .join(env!("CARGO_PKG_VERSION"));
    dirs_to_write.push(cache_dir.clone());
    // Make typedef files
    for (builtin_name, builtin_typedef) in typedef_files {
        let path = cache_dir
            .join(builtin_name.to_ascii_lowercase())
            .with_extension("luau");
        files_to_write.push((builtin_name.to_lowercase(), path, builtin_typedef));
    }
    // Write all dirs and files only when we know generation was successful
    let futs_dirs = dirs_to_write
        .drain(..)
        .map(create_dir_all)
        .collect::<Vec<_>>();
    let futs_files = files_to_write
        .iter()
        .map(|(_, path, contents)| write(path, contents))
        .collect::<Vec<_>>();
    try_join_all(futs_dirs).await?;
    try_join_all(futs_files).await?;
    Ok(files_to_write
        .drain(..)
        .map(|(name, path, _)| (name, path))
        .collect::<HashMap<_, _>>())
}
