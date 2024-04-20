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

pub struct LuaCaptures {
    inner: LuaCapturesInner,
}

impl LuaCaptures {
    pub fn new(pattern: &Regex, text: String) -> Self {
        Self {
            inner: LuaCapturesInner::new(Arc::from(text), |owned| pattern.captures(owned.as_str())),
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
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("get", |_, this, n: usize| {
            Ok(this
                .captures()
                .get(n)
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

        methods.add_meta_method(LuaMetaMethod::Type, |_, _, ()| Ok("RegexCaptures"));
        methods.add_meta_method(LuaMetaMethod::Len, |_, this, ()| Ok(this.num_captures()));
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
            Ok(format!("RegexCaptures({})", this.num_captures()))
        });
    }
}
