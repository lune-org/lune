use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum DocumentError {
    #[error("Attempted to read or write internal root document")]
    InternalRootReadWrite,
    #[error("Unknown document kind")]
    UnknownKind,
    #[error("Unknown document format")]
    UnknownFormat,
    #[error("Failed to read document from buffer")]
    ReadError(String),
    #[error("Failed to write document to buffer")]
    WriteError(String),
}
