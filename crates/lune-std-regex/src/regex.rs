use std::sync::Arc;

use mlua::prelude::*;
use regex::Regex;

use super::{captures::LuaCaptures, matches::LuaMatch};

/**
    A wrapper over the `regex::Regex` struct that can be used from Lua.
*/
#[derive(Debug, Clone)]
pub struct LuaRegex {
    inner: Regex,
}

impl LuaRegex {
    /**
        Create a new `LuaRegex` instance from a `String` pattern.
    */
    pub fn new(pattern: String) -> LuaResult<Self> {
        Regex::new(&pattern)
            .map(|inner| Self { inner })
            .map_err(LuaError::external)
    }
}

impl LuaUserData for LuaRegex {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("isMatch", |_, this, text: String| {
            Ok(this.inner.is_match(&text))
        });

        methods.add_method("find", |_, this, text: String| {
            let arc = Arc::new(text);
            Ok(this
                .inner
                .find(&arc)
                .map(|m| LuaMatch::new(Arc::clone(&arc), m)))
        });

        methods.add_method("captures", |_, this, text: String| {
            Ok(LuaCaptures::new(&this.inner, text))
        });

        methods.add_method("split", |_, this, text: String| {
            Ok(this
                .inner
                .split(&text)
                .map(ToString::to_string)
                .collect::<Vec<_>>())
        });

        // TODO: Determine whether it's desirable and / or feasible to support
        // using a function or table for `replace` like in the lua string library
        methods.add_method(
            "replace",
            |_, this, (haystack, replacer): (String, String)| {
                Ok(this.inner.replace(&haystack, replacer).to_string())
            },
        );
        methods.add_method(
            "replaceAll",
            |_, this, (haystack, replacer): (String, String)| {
                Ok(this.inner.replace_all(&haystack, replacer).to_string())
            },
        );

        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            Ok(this.inner.as_str().to_string())
        });
    }

    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_meta_field(LuaMetaMethod::Type, "Regex");
    }
}
