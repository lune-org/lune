use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use tokio::fs::{read_to_string, remove_file};

use super::bin_dir::{enter_bin_dir, leave_bin_dir};
use super::file_type::FileType;

pub fn fmt_path_relative_to_workspace_root(value: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .unwrap();
    match PathBuf::from(value).strip_prefix(root) {
        Err(_) => format!("{:#?}", PathBuf::from(value).display()),
        Ok(inner) => format!("{:#?}", inner.display()),
    }
}

async fn inner(file_name: &str, desired_type: FileType) -> Result<()> {
    match read_to_string(file_name).await.with_context(|| {
        format!(
            "Failed to read definitions file at '{}'",
            fmt_path_relative_to_workspace_root(file_name)
        )
    }) {
        Ok(file_contents) => {
            remove_file(file_name).await.with_context(|| {
                format!(
                    "Failed to remove definitions file at '{}'",
                    fmt_path_relative_to_workspace_root(file_name)
                )
            })?;
            let parsed_type = FileType::from_contents(&file_contents);
            if parsed_type != Some(desired_type) {
                bail!(
                    "Generating definitions file at '{}' created '{}', expected '{}'",
                    fmt_path_relative_to_workspace_root(file_name),
                    parsed_type.map_or("unknown", |t| t.name()),
                    desired_type.name()
                )
            }
            Ok(())
        }
        Err(e) => bail!(
            "Failed to generate definitions file at '{}'\n{e}",
            fmt_path_relative_to_workspace_root(file_name)
        ),
    }
}

pub async fn ensure_file_exists_and_is(file_name: &str, desired_type: FileType) -> Result<()> {
    enter_bin_dir().await?;
    let res = inner(file_name, desired_type).await;
    leave_bin_dir()?;
    res
}
