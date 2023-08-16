use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

use mlua::prelude::*;

use crate::lune_temp::lua::stdio::formatting::pretty_format_luau_error;

/**
    An opaque error type for formatted lua errors.
*/
#[derive(Debug, Clone)]
pub struct LuneError {
    error: LuaError,
    disable_colors: bool,
}

impl LuneError {
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

impl From<LuaError> for LuneError {
    fn from(value: LuaError) -> Self {
        Self {
            error: value,
            disable_colors: false,
        }
    }
}

impl Display for LuneError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            pretty_format_luau_error(&self.error, !self.disable_colors)
        )
    }
}

impl Error for LuneError {
    // TODO: Comment this out when we are ready to also re-export
    // `mlua` as part of our public library interface in Lune
    // fn cause(&self) -> Option<&dyn Error> {
    //     Some(&self.error)
    // }
}
