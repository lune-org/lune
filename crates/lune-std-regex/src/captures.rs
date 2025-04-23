use std::sync::Arc;

use mlua::prelude::*;
use regex::{Captures, Regex};
use self_cell::self_cell;

use super::matches::LuaMatch;

type OptionalCaptures<'a> = Option<Captures<'a>>;

self_cell! {
    struct LuaCapturesInner {
        owner: Arc<String>,
        #[covariant]
        dependent: OptionalCaptures,
    }
}

/**
    A wrapper over the `regex::Captures` struct that can be used from Lua.
*/
pub struct LuaCaptures {
    inner: LuaCapturesInner,
}

impl LuaCaptures {
    /**
        Create a new `LuaCaptures` instance from a `Regex` pattern and a `String` text.

        Returns `Some(_)` if captures were found, `None` if no captures were found.
    */
    pub fn new(pattern: &Regex, text: String) -> Option<Self> {
        let inner =
            LuaCapturesInner::new(Arc::from(text), |owned| pattern.captures(owned.as_str()));
        if inner.borrow_dependent().is_some() {
            Some(Self { inner })
        } else {
            None
        }
    }

    fn captures(&self) -> &Captures {
        self.inner
            .borrow_dependent()
            .as_ref()
            .expect("None captures should not be used")
    }

    fn num_captures(&self) -> usize {
        // NOTE: Here we exclude the match for the entire regex
        // pattern, only counting the named and numbered captures
        self.captures().len() - 1
    }

    fn text(&self) -> Arc<String> {
        Arc::clone(self.inner.borrow_owner())
    }
}

impl LuaUserData for LuaCaptures {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get", |_, this, index: usize| {
            Ok(this
                .captures()
                .get(index)
                .map(|m| LuaMatch::new(this.text(), m)))
        });

        methods.add_method("group", |_, this, group: String| {
            Ok(this
                .captures()
                .name(&group)
                .map(|m| LuaMatch::new(this.text(), m)))
        });

        methods.add_method("format", |_, this, format: String| {
            let mut new = String::new();
            this.captures().expand(&format, &mut new);
            Ok(new)
        });

        methods.add_meta_method(LuaMetaMethod::Len, |_, this, ()| Ok(this.num_captures()));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            Ok(format!("{}", this.num_captures()))
        });
    }

    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_meta_field(LuaMetaMethod::Type, "RegexCaptures");
    }
}
