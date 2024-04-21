use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use mlua::prelude::*;

use crate::lune::util::formatting::pretty_format_luau_error;

/**
    An opaque error type for formatted lua errors.
*/
#[derive(Debug, Clone)]
pub struct RuntimeError {
    error: LuaError,
    disable_colors: bool,
}

impl RuntimeError {
    /**
        Enables colorization of the error message when formatted using the [`Display`] trait.

        Colorization is enabled by default.
    */
    #[doc(hidden)]
    pub fn enable_colors(mut self) -> Self {
        self.disable_colors = false;
        self
    }

    /**
        Disables colorization of the error message when formatted using the [`Display`] trait.

        Colorization is enabled by default.
    */
    #[doc(hidden)]
    pub fn disable_colors(mut self) -> Self {
        self.disable_colors = true;
        self
    }

    /**
        Returns `true` if the error can likely be fixed by appending more input to the source code.

        See [`mlua::Error::SyntaxError`] for more information.
    */
    pub fn is_incomplete_input(&self) -> bool {
        matches!(
            self.error,
            LuaError::SyntaxError {
                incomplete_input: true,
                ..
            }
        )
    }
}

impl From<LuaError> for RuntimeError {
    fn from(value: LuaError) -> Self {
        Self {
            error: value,
            disable_colors: false,
        }
    }
}

impl From<&LuaError> for RuntimeError {
    fn from(value: &LuaError) -> Self {
        Self {
            error: value.clone(),
            disable_colors: false,
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            pretty_format_luau_error(&self.error, !self.disable_colors)
        )
    }
}

impl Error for RuntimeError {
    fn cause(&self) -> Option<&dyn Error> {
        Some(&self.error)
    }
}
