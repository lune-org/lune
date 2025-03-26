use std::{collections::HashSet, sync::Arc};

use console::{colors_enabled as get_colors_enabled, set_colors_enabled};
use mlua::prelude::*;
use once_cell::sync::Lazy;
use parking_lot::ReentrantMutex;

mod basic;
mod config;
mod metamethods;
mod recursive;
mod style;

use self::recursive::format_value_recursive;

pub use self::config::ValueFormatConfig;

// NOTE: Since the setting for colors being enabled is global,
// and these functions may be called in parallel, we use this global
// lock to make sure that we don't mess up the colors for other threads.
static COLORS_LOCK: Lazy<Arc<ReentrantMutex<()>>> = Lazy::new(|| Arc::new(ReentrantMutex::new(())));

/**
    Formats a Lua value into a pretty string using the given config.
*/
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn pretty_format_value(value: &LuaValue, config: &ValueFormatConfig) -> String {
    let _guard = COLORS_LOCK.lock();

    let were_colors_enabled = get_colors_enabled();
    set_colors_enabled(were_colors_enabled && config.colors_enabled);

    let mut visited = HashSet::new();
    let res = format_value_recursive(value, config, &mut visited, 0);

    set_colors_enabled(were_colors_enabled);
    res.expect("using fmt for writing into strings should never fail")
}

/**
    Formats a Lua multi-value into a pretty string using the given config.

    Each value will be separated by a space.
*/
#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn pretty_format_multi_value(values: &LuaMultiValue, config: &ValueFormatConfig) -> String {
    let _guard = COLORS_LOCK.lock();

    let were_colors_enabled = get_colors_enabled();
    set_colors_enabled(were_colors_enabled && config.colors_enabled);

    let mut visited = HashSet::new();
    let res = values
        .into_iter()
        .map(|value| format_value_recursive(value, config, &mut visited, 0))
        .collect::<Result<Vec<_>, _>>();

    set_colors_enabled(were_colors_enabled);
    res.expect("using fmt for writing into strings should never fail")
        .join(" ")
}
