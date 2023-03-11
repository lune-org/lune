use core::fmt;

use std::error::Error;
use std::io::Error as IoError;

use mlua::Error as LuaError;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum DatatypeConversionError {
    LuaError(LuaError),
    External {
        message: String,
    },
    FromRbxVariant {
        from: &'static str,
        to: &'static str,
        detail: Option<String>,
    },
    ToRbxVariant {
        to: &'static str,
        from: &'static str,
        detail: Option<String>,
    },
}

impl fmt::Display for DatatypeConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::LuaError(error) => error.to_string(),
                Self::External { message } => message.to_string(),
                Self::FromRbxVariant { from, to, detail }
                | Self::ToRbxVariant { from, to, detail } => {
                    match detail {
                        Some(d) => format!("Failed to convert from '{from}' into '{to}' - {d}"),
                        None => format!("Failed to convert from '{from}' into '{to}'",),
                    }
                }
            }
        )
    }
}

impl Error for DatatypeConversionError {}

impl From<LuaError> for DatatypeConversionError {
    fn from(value: LuaError) -> Self {
        Self::LuaError(value)
    }
}

impl From<IoError> for DatatypeConversionError {
    fn from(value: IoError) -> Self {
        DatatypeConversionError::External {
            message: value.to_string(),
        }
    }
}

impl From<base64::DecodeError> for DatatypeConversionError {
    fn from(value: base64::DecodeError) -> Self {
        DatatypeConversionError::External {
            message: value.to_string(),
        }
    }
}

pub(crate) type DatatypeConversionResult<T> = Result<T, DatatypeConversionError>;
