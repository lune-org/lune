use std::{ops::Range, sync::Arc};

use mlua::prelude::*;
use regex::Match;

pub struct LuaMatch {
    text: Arc<String>,
    start: usize,
    end: usize,
}

impl LuaMatch {
    pub fn new(text: Arc<String>, matched: Match) -> Self {
        Self {
            text,
            start: matched.start(),
            end: matched.end(),
        }
    }

    fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    fn slice(&self) -> &str {
        &self.text[self.range()]
    }
}

impl LuaUserData for LuaMatch {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        // NOTE: Strings are 0 based in Rust but 1 based in Luau, and end of range in Rust is exclusive
        fields.add_field_method_get("start", |_, this| Ok(this.start.saturating_add(1)));
        fields.add_field_method_get("finish", |_, this| Ok(this.end));
        fields.add_field_method_get("len", |_, this| Ok(this.range().len()));
        fields.add_field_method_get("text", |_, this| Ok(this.slice().to_string()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("isEmpty", |_, this, ()| Ok(this.range().is_empty()));

        methods.add_meta_method(LuaMetaMethod::Type, |_, _, ()| Ok("RegexMatch"));
        methods.add_meta_method(LuaMetaMethod::Len, |_, this, ()| Ok(this.range().len()));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            Ok(format!("RegexMatch({})", this.slice()))
        });
    }
}
