use thiserror::Error;

use super::target::BuildTarget;

/**
    Errors that may occur when building a standalone binary
*/
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("failed to find lune target '{0}' in GitHub release")]
    ReleaseTargetNotFound(BuildTarget),
    #[error("failed to find lune binary '{0}' in downloaded zip file")]
    ZippedBinaryNotFound(String),
    #[error("failed to download lune binary: {0}")]
    Download(#[from] reqwest::Error),
    #[error("failed to unzip lune binary: {0}")]
    Unzip(#[from] zip::result::ZipError),
    #[error("panicked while unzipping lune binary: {0}")]
    UnzipJoin(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type BuildResult<T, E = BuildError> = std::result::Result<T, E>;
