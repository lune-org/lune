use std::{env::set_current_dir, path::PathBuf};

use anyhow::{Context, Result};
use tokio::fs::create_dir_all;

pub async fn enter_bin_dir() -> Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bin");
    if !path.exists() {
        create_dir_all(&path)
            .await
            .context("Failed to enter bin dir")?;
        set_current_dir(&path).context("Failed to set current dir")?;
    }
    Ok(())
}

pub fn leave_bin_dir() -> Result<()> {
    set_current_dir(env!("CARGO_MANIFEST_DIR")).context("Failed to leave bin dir")?;
    Ok(())
}
