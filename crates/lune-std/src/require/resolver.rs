use std::{
    fs::read as read_file,
    io::Result as IoResult,
    path::{Path, PathBuf},
};

use lune_utils::path::clean_path_and_make_absolute;
use mlua::prelude::*;

use crate::require::loader::RequireLoader;

use super::{
    constants::{FILE_CHUNK_PREFIX, FILE_NAME_CONFIG},
    path_utils::{relative_path_normalize, relative_path_parent},
    resolved_path::ResolvedPath,
};

#[derive(Debug)]
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
    /// Loader and accompanying state.
    loader: RequireLoader,
}

impl RequireResolver {
    pub(crate) fn new() -> Self {
        Self {
            relative: PathBuf::new(),
            absolute: PathBuf::new(),
            resolved: None,
            loader: RequireLoader::new(),
        }
    }

    fn navigate_reset(&mut self) {
        self.relative = PathBuf::new();
        self.absolute = PathBuf::new();
        self.resolved = None;
    }

    fn navigate_to(
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
        let resolved = ResolvedPath::resolve(&absolute)?;

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
        // NOTE: This is not actually necessary, but makes our resolver state
        // behave a bit more consistently - it ensures `resolved == None` when
        // no file has been resolved from the current module path navigation.
        // It is really only useful when debugging the require resolver state.
        self.navigate_reset();

        if let Some(path) = chunk_name.strip_prefix(FILE_CHUNK_PREFIX) {
            let rel = relative_path_normalize(Path::new(path));
            let abs = clean_path_and_make_absolute(&rel);

            self.navigate_to(rel, abs)
        } else {
            Err(LuaNavigateError::Other(LuaError::runtime(
                "cannot reset require state from non-file chunk",
            )))
        }
    }

    fn jump_to_alias(&mut self, path: &str) -> Result<(), LuaNavigateError> {
        let rel = relative_path_normalize(Path::new(path));
        let abs = clean_path_and_make_absolute(&rel);

        self.navigate_to(rel, abs)
    }

    fn to_parent(&mut self) -> Result<(), LuaNavigateError> {
        let mut rel = self.relative.clone();
        let mut abs = self.absolute.clone();

        if !abs.pop() {
            return Err(LuaNavigateError::Other(LuaError::runtime(
                "tried to require a module at the filesystem root",
            )));
        }

        relative_path_parent(&mut rel);

        self.navigate_to(rel, abs)
    }

    fn to_child(&mut self, name: &str) -> Result<(), LuaNavigateError> {
        let rel = self.relative.join(name);
        let abs = self.absolute.join(name);

        self.navigate_to(rel, abs)
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
        let resolved = resolved.as_file().expect("tried to require a dir");
        self.loader.load(lua, self.relative.as_path(), resolved)
    }
}
