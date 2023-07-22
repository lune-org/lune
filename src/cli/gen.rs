use std::collections::HashMap;

use anyhow::{Context, Result};
use directories::UserDirs;
use futures_util::future::try_join_all;
use include_dir::Dir;
use tokio::fs::{create_dir_all, write};

pub async fn generate_typedef_files_from_definitions(dir: &Dir<'_>) -> Result<String> {
    let contents = read_typedefs_dir_contents(dir);
    write_typedef_files(contents).await
}

fn read_typedefs_dir_contents(dir: &Dir<'_>) -> HashMap<String, Vec<u8>> {
    let mut definitions = HashMap::new();

    for entry in dir.find("*.luau").unwrap() {
        let entry_file = entry.as_file().unwrap();
        let entry_name = entry_file.path().file_name().unwrap().to_string_lossy();

        let typedef_name = entry_name.trim_end_matches(".luau");
        let typedef_contents = entry_file.contents().to_vec();

        definitions.insert(typedef_name.to_string(), typedef_contents);
    }

    definitions
}

async fn write_typedef_files(typedef_files: HashMap<String, Vec<u8>>) -> Result<String> {
    let version_string = env!("CARGO_PKG_VERSION");
    let mut dirs_to_write = Vec::new();
    let mut files_to_write = Vec::new();
    // Create the typedefs dir in the users cache dir
    let cache_dir = UserDirs::new()
        .context("Failed to find user home directory")?
        .home_dir()
        .join(".lune")
        .join(".typedefs")
        .join(version_string);
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
    Ok(version_string.to_string())
}
