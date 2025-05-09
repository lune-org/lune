use std::path::{Path, PathBuf};

use anyhow::Result;
use async_fs as fs;
use futures_lite::prelude::*;

/**
    Removes the source file extension from the given path, if it has one.

    A source file extension is an extension such as `.lua` or `.luau`.
*/
pub fn remove_source_file_ext(path: &Path) -> PathBuf {
    if path
        .extension()
        .is_some_and(|ext| matches!(ext.to_str(), Some("lua" | "luau")))
    {
        path.with_extension("")
    } else {
        path.to_path_buf()
    }
}

/**
    Writes the given bytes to a file at the specified path,
    and makes sure it has permissions to be executed.
*/
pub async fn write_executable_file_to(
    path: impl AsRef<Path>,
    bytes: impl AsRef<[u8]>,
) -> Result<(), std::io::Error> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    {
        use fs::unix::OpenOptionsExt;
        options.mode(0o755); // Read & execute for all, write for owner
    }

    let mut file = options.open(path).await?;
    file.write_all(bytes.as_ref()).await?;
    file.flush().await?;

    Ok(())
}
