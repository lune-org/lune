use std::{
    fs::read as read_file,
    io::Result as IoResult,
    path::{Component, PathBuf},
};

use lune_utils::path::{absolute_path, clean_path};
use mlua::prelude::*;

use super::{
    constants::{FILE_CHUNK_PREFIX, FILE_NAME_CONFIG},
    resolved_path::ResolvedPath,
};

#[derive(Default, Debug)]
pub(crate) struct RequireResolver {
    /// Path to the current module, absolute.
    ///
    /// Not guaranteed to be a valid filesystem
    /// path, and should not be used as such.
    absolute: PathBuf,
    /// Path to the current module, relative.
    ///
    /// Not guaranteed to be a valid filesystem
    /// path, and should not be used as such.
    relative: PathBuf,
    /// Path to the current filesystem entry that
    /// directly represents the current module path.
    resolved: Option<ResolvedPath>,
}

impl RequireResolver {
    fn update_paths(
        &mut self,
        relative: PathBuf,
        absolute: PathBuf,
    ) -> Result<(), LuaNavigateError> {
        // Avoid unnecessary filesystem calls when possible - ResolvedPath
        // should always point to the same filesystem entry as long as the
        // relative and absolute module paths stay the same.
        if self.relative == relative && self.absolute == absolute {
            return Ok(());
        }

        // Make sure to resolve path **before** updating any paths state
        let resolved = ResolvedPath::resolve(&relative)?;

        self.absolute = absolute;
        self.relative = relative;
        self.resolved = Some(resolved);

        Ok(())
    }
}

impl LuaRequire for RequireResolver {
    fn is_require_allowed(&self, chunk_name: &str) -> bool {
        chunk_name.starts_with(FILE_CHUNK_PREFIX)
    }

    fn reset(&mut self, chunk_name: &str) -> Result<(), LuaNavigateError> {
        if let Some(path) = chunk_name.strip_prefix(FILE_CHUNK_PREFIX) {
            let rel = clean_path(path);
            let abs = absolute_path(&rel);
            self.update_paths(rel, abs)
        } else {
            Err(LuaNavigateError::Other(LuaError::runtime(
                "cannot reset require state from non-file chunk",
            )))
        }
    }

    fn jump_to_alias(&mut self, path: &str) -> Result<(), LuaNavigateError> {
        let rel = clean_path(path);
        let abs = absolute_path(&rel);

        self.update_paths(rel, abs)
    }

    fn to_parent(&mut self) -> Result<(), LuaNavigateError> {
        let mut rel = self.relative.clone();
        let mut abs = self.absolute.clone();

        if !abs.pop() {
            return Err(LuaNavigateError::Other(LuaError::runtime(
                "tried to require a module at the filesystem root",
            )));
        }

        // If our relative path becomes empty, we should keep traversing it,
        // but we need to do so by appending the special "parent dir" component,
        // which is normally represented by ".."
        if rel.components().all(|c| matches!(c, Component::ParentDir)) {
            rel.push(Component::ParentDir);
        } else {
            rel.pop();
        }

        self.update_paths(rel, abs)
    }

    fn to_child(&mut self, name: &str) -> Result<(), LuaNavigateError> {
        let rel = self.relative.join(name);
        let abs = self.absolute.join(name);

        self.update_paths(rel, abs)
    }

    fn has_module(&self) -> bool {
        let resolved = self.resolved.as_ref();
        resolved.is_some_and(ResolvedPath::is_file)
    }

    fn cache_key(&self) -> String {
        let resolved = self.resolved.as_ref();
        resolved.expect("called has_module first").to_string()
    }

    fn has_config(&self) -> bool {
        self.absolute.is_dir() && self.absolute.join(FILE_NAME_CONFIG).is_file()
    }

    fn config(&self) -> IoResult<Vec<u8>> {
        read_file(self.absolute.join(FILE_NAME_CONFIG))
    }

    fn loader(&self, lua: &Lua) -> LuaResult<LuaFunction> {
        let resolved = self.resolved.as_ref();
        let resolved = resolved.expect("called has_module first");

        let name = format!("{FILE_CHUNK_PREFIX}{}", self.relative.display());
        let bytes = read_file(resolved)?;

        lua.load(bytes).set_name(name).into_function()
    }
}
