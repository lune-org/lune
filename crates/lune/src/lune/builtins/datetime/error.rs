use mlua::prelude::*;

use thiserror::Error;

pub type DateTimeResult<T, E = DateTimeError> = Result<T, E>;

#[derive(Debug, Clone, Error)]
pub enum DateTimeError {
    #[error("invalid date")]
    InvalidDate,
    #[error("invalid time")]
    InvalidTime,
    #[error("ambiguous date or time")]
    Ambiguous,
    #[error("date or time is outside allowed range")]
    OutOfRangeUnspecified,
    #[error("{name} must be within range {min} -> {max}, got {value}")]
    OutOfRange {
        name: &'static str,
        value: String,
        min: String,
        max: String,
    },
    #[error(transparent)]
    ParseError(#[from] chrono::ParseError),
}

impl From<DateTimeError> for LuaError {
    fn from(value: DateTimeError) -> Self {
        LuaError::runtime(value.to_string())
    }
}
