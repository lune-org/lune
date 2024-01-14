use mlua::prelude::*;

use super::formatting::format_label;
use crate::RuntimeError;

pub trait LuaEmitErrorExt {
    fn emit_error(&self, err: LuaError);
}

impl LuaEmitErrorExt for Lua {
    fn emit_error(&self, err: LuaError) {
        // NOTE: LuneError will pretty-format this error
        eprintln!("{}\n{}", format_label("error"), RuntimeError::from(err));
    }
}
