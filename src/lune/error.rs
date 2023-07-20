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
}

#[allow(dead_code)]
impl LuneError {
    pub(crate) fn new(message: String) -> Self {
        Self { message }
    }

    pub(crate) fn from_lua_error(error: LuaError) -> Self {
        Self::new(pretty_format_luau_error(&error, true))
    }

    pub(crate) fn from_lua_error_plain(error: LuaError) -> Self {
        Self::new(pretty_format_luau_error(&error, false))
    }
}

impl Display for LuneError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message)
    }
}

impl Error for LuneError {}
