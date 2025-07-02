use std::{
    ffi::OsStr,
    fmt,
    path::{Path, PathBuf},
};

use mlua::prelude::*;

use super::constants::{FILE_EXTENSIONS, FILE_NAME_INIT};

/**
    A module path resolved to either a valid file or directory.

    See [`ResolvedPath::resolve`] for more information.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedPath {
    /// A resolved and valid file path.
    File(PathBuf),
    /// A resolved and valid directory path.
    Directory(PathBuf),
}

impl ResolvedPath {
    /**
        Resolves an existing file or directory path for the given *module* path.

        Given a *module* path "path/to/module", these files will be searched:

        - `path/to/module.luau`
        - `path/to/module.lua`
        - `path/to/module/init.luau`
        - `path/to/module/init.lua`

        If the given path ("path/to/module") is a directory instead,
        and it exists, it will be returned without any modifications.
    */
    pub(crate) fn resolve(path: &Path) -> Result<ResolvedPath, LuaNavigateError> {
        // Modules named "init" are ambiguous and not allowed
        if path
            .file_name()
            .is_some_and(|n| n == OsStr::new(FILE_NAME_INIT))
        {
            return Err(LuaNavigateError::Ambiguous);
        }

        let mut found = None;

        // Try files first
        for ext in FILE_EXTENSIONS {
            let candidate = append_extension(path, ext);
            if candidate.is_file() && found.replace(candidate).is_some() {
                return Err(LuaNavigateError::Ambiguous);
            }
        }

        // Try directories with init files in them
        if path.is_dir() {
            let init = Path::new(FILE_NAME_INIT);
            for ext in FILE_EXTENSIONS {
                let candidate = path.join(append_extension(init, ext));
                if candidate.is_file() && found.replace(candidate).is_some() {
                    return Err(LuaNavigateError::Ambiguous);
                }
            }

            // If we have not found any luau / lua files, and we also did not find
            // any init files in this directory, we still found a valid directory
            if found.is_none() {
                return Ok(ResolvedPath::Directory(path.to_path_buf()));
            }
        }

        // We have now narrowed down our resulting module
        // path to be exactly one valid path, or no path
        found
            .map(ResolvedPath::File)
            .ok_or(LuaNavigateError::NotFound)
    }

    pub(crate) const fn is_file(&self) -> bool {
        matches!(self, ResolvedPath::File(_))
    }
}

impl AsRef<Path> for ResolvedPath {
    fn as_ref(&self) -> &Path {
        match self {
            ResolvedPath::File(path) | ResolvedPath::Directory(path) => path.as_ref(),
        }
    }
}

impl fmt::Display for ResolvedPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().display().fmt(f)
    }
}

fn append_extension(path: &Path, ext: &str) -> PathBuf {
    match path.extension() {
        None => path.with_extension(ext),
        Some(curr_ext) => {
            let mut new_ext = curr_ext.to_os_string();
            new_ext.push(".");
            new_ext.push(ext);
            path.with_extension(new_ext)
        }
    }
}
