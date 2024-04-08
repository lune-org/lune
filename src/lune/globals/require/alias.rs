use console::style;
use mlua::prelude::*;

use crate::lune::util::{
    luaurc::LuauRc,
    paths::{make_absolute_and_clean, CWD},
};

use super::context::*;

pub(super) async fn require<'lua, 'ctx>(
    lua: &'lua Lua,
    ctx: &'ctx RequireContext,
    source: &str,
    alias: &str,
    path: &str,
) -> LuaResult<LuaMultiValue<'lua>>
where
    'lua: 'ctx,
{
    let alias = alias.to_ascii_lowercase();

    let parent = make_absolute_and_clean(source)
        .parent()
        .expect("how did a root path end up here..")
        .to_path_buf();

    // Try to gather the first luaurc and / or error we
    // encounter to display better error messages to users
    let mut first_luaurc = None;
    let mut first_error = None;
    let predicate = |rc: &LuauRc| {
        if first_luaurc.is_none() {
            first_luaurc.replace(rc.clone());
        }
        if let Err(e) = rc.validate() {
            if first_error.is_none() {
                first_error.replace(e);
            }
            false
        } else {
            rc.find_alias(&alias).is_some()
        }
    };

    // Try to find a luaurc that contains the alias we're searching for
    let luaurc = LuauRc::read_recursive(parent, predicate)
        .await
        .ok_or_else(|| {
            if let Some(error) = first_error {
                LuaError::runtime(format!("error while parsing .luaurc file: {error}"))
            } else if let Some(luaurc) = first_luaurc {
                LuaError::runtime(format!(
                    "failed to find alias '{alias}' - known aliases:\n{}",
                    luaurc
                        .aliases()
                        .iter()
                        .map(|(name, path)| format!("    {name} {} {path}", style(">").dim()))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            } else {
                LuaError::runtime(format!("failed to find alias '{alias}' (no .luaurc)"))
            }
        })?;

    // We now have our aliased path, our path require function just needs it
    // in a slightly different format with both absolute + relative to cwd
    let abs_path = luaurc.find_alias(&alias).unwrap().join(path);
    let rel_path = pathdiff::diff_paths(&abs_path, CWD.as_path()).ok_or_else(|| {
        LuaError::runtime(format!("failed to find relative path for alias '{alias}'"))
    })?;

    super::path::require_abs_rel(lua, ctx, abs_path, rel_path).await
}
