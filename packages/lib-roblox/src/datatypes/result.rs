use core::fmt;

use std::error::Error;
use std::io::Error as IoError;

use mlua::Error as LuaError;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum DomConversionError {
    LuaError(LuaError),
    External {
        message: String,
    },
    FromDomValue {
        from: &'static str,
        to: &'static str,
        detail: Option<String>,
    },
    ToDomValue {
        to: &'static str,
        from: &'static str,
        detail: Option<String>,
    },
}

impl fmt::Display for DomConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::LuaError(error) => error.to_string(),
                Self::External { message } => message.to_string(),
                Self::FromDomValue { from, to, detail } | Self::ToDomValue { from, to, detail } => {
                    match detail {
                        Some(d) => format!("Failed to convert from '{from}' into '{to}' - {d}"),
                        None => format!("Failed to convert from '{from}' into '{to}'",),
                    }
                }
            }
        )
    }
}

impl Error for DomConversionError {}

impl From<DomConversionError> for LuaError {
    fn from(value: DomConversionError) -> Self {
        use DomConversionError as E;
        match value {
            E::LuaError(e) => e,
            E::External { message } => LuaError::external(message),
            E::FromDomValue { .. } | E::ToDomValue { .. } => {
                LuaError::RuntimeError(value.to_string())
            }
        }
    }
}

impl From<LuaError> for DomConversionError {
    fn from(value: LuaError) -> Self {
        Self::LuaError(value)
    }
}

impl From<IoError> for DomConversionError {
    fn from(value: IoError) -> Self {
        DomConversionError::External {
            message: value.to_string(),
        }
    }
}

pub(crate) type DomConversionResult<T> = Result<T, DomConversionError>;
