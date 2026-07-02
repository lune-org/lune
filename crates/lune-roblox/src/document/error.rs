use thiserror::Error;

#[cfg(feature = "mlua")]
use mlua::prelude::*;

#[derive(Debug, Clone, Error)]
pub enum DocumentError {
    #[error("Unknown document kind")]
    UnknownKind,
    #[error("Unknown document format")]
    UnknownFormat,
    #[error("Failed to read document from buffer - {0}")]
    ReadError(String),
    #[error("Failed to write document to buffer - {0}")]
    WriteError(String),
    #[error("Failed to convert into a DataModel - the given document is not a place")]
    IntoDataModelInvalidArgs,
    #[error("Failed to convert into array of Instances - the given document is a model")]
    IntoInstanceArrayInvalidArgs,
    #[error("Failed to convert into a place - the given instance is not a DataModel")]
    FromDataModelInvalidArgs,
    #[error("Failed to convert into a model - a given instance is a DataModel")]
    FromInstanceArrayInvalidArgs,
}

#[cfg(feature = "mlua")]
impl From<DocumentError> for LuaError {
    fn from(value: DocumentError) -> Self {
        Self::RuntimeError(value.to_string())
    }
}
