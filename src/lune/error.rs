use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use mlua::prelude::*;

use crate::lune::lua::stdio::formatting::pretty_format_luau_error;

/**
    An opaque error type for formatted lua errors.
*/
#[derive(Debug, Clone)]
pub struct LuneError {
    message: String,
    incomplete_input: bool,
}

impl LuneError {
    pub(crate) fn from_lua_error(error: LuaError, disable_colors: bool) -> Self {
        Self {
            message: pretty_format_luau_error(&error, !disable_colors),
            incomplete_input: matches!(
                error,
                LuaError::SyntaxError {
                    incomplete_input: true,
                    ..
                }
            ),
        }
    }
}

impl LuneError {
    pub fn is_incomplete_input(&self) -> bool {
        self.incomplete_input
    }
}

impl Display for LuneError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message)
    }
}

impl Error for LuneError {}
