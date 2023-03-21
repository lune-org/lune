use mlua::prelude::*;
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
    #[error("Failed to convert into a DataModel - the given document is not a place")]
    IntoDataModelInvalidArgs,
    #[error("Failed to convert into array of Instances - the given document is a place")]
    IntoInstanceArrayInvalidArgs,
    #[error("Failed to convert into a document - the given instance is not a DataModel")]
    FromDataModelInvalidArgs,
    #[error("Failed to convert into a document - a given instances is a DataModel")]
    FromInstanceArrayInvalidArgs,
}

impl From<DocumentError> for LuaError {
    fn from(value: DocumentError) -> Self {
        Self::RuntimeError(value.to_string())
    }
}
