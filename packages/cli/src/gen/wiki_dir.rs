use std::{fmt::Write, path::PathBuf};

use anyhow::{Context, Result};

use tokio::fs::{create_dir_all, write};

use super::definitions::DefinitionsTree;

pub const GENERATED_COMMENT_TAG: &str = "@generated with lune-cli";

pub async fn generate_from_type_definitions(contents: &str) -> Result<()> {
    let tree = DefinitionsTree::from_type_definitions(contents)?;
    // Create the wiki dir at the repo root
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .unwrap();
    create_dir_all(&root.join("wiki"))
        .await
        .context("Failed to create wiki dir")?;
    Ok(())
}
