/*!
    Utilities for working with Luau module paths.
*/

use std::{
    ffi::OsStr,
    fmt,
    path::{Path, PathBuf},
};

use mlua::prelude::*;

use super::constants::{FILE_EXTENSIONS, FILE_NAME_INIT};
use super::std::append_extension;

/**
    A file path for Luau, which has been resolved to either a valid file or directory.

    Not to be confused with [`LuauModulePath`]. This is the path
    **on the filesystem**, and not the abstracted module path.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LuauFilePath {
    /// A resolved and valid file path.
    File(PathBuf),
    /// A resolved and valid directory path.
    Directory(PathBuf),
}

impl LuauFilePath {
    fn resolve(module: impl AsRef<Path>) -> Result<Self, LuaNavigateError> {
        let module = module.as_ref();

        // Modules named "init" are ambiguous and not allowed
        if module
            .file_name()
            .is_some_and(|n| n == OsStr::new(FILE_NAME_INIT))
        {
            return Err(LuaNavigateError::Ambiguous);
        }

        let mut found = None;

        // Try files first
        for ext in FILE_EXTENSIONS {
            let candidate = append_extension(module, ext);
            if candidate.is_file() && found.replace(candidate).is_some() {
                return Err(LuaNavigateError::Ambiguous);
            }
        }

        // Try directories with init files in them
        if module.is_dir() {
            let init = Path::new(FILE_NAME_INIT);
            for ext in FILE_EXTENSIONS {
                let candidate = module.join(append_extension(init, ext));
                if candidate.is_file() && found.replace(candidate).is_some() {
                    return Err(LuaNavigateError::Ambiguous);
                }
            }

            // If we have not found any luau / lua files, and we also did not find
            // any init files in this directory, we still found a valid directory
            if found.is_none() {
                return Ok(Self::Directory(module.to_path_buf()));
            }
        }

        // We have now narrowed down our resulting module
        // path to be exactly one valid path, or no path
        found.map(Self::File).ok_or(LuaNavigateError::NotFound)
    }

    #[must_use]
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self, Self::Directory(_))
    }

    #[must_use]
    pub fn as_file(&self) -> Option<&Path> {
        match self {
            Self::File(path) => Some(path),
            Self::Directory(_) => None,
        }
    }

    #[must_use]
    pub fn as_dir(&self) -> Option<&Path> {
        match self {
            Self::File(_) => None,
            Self::Directory(path) => Some(path),
        }
    }
}

impl AsRef<Path> for LuauFilePath {
    fn as_ref(&self) -> &Path {
        match self {
            Self::File(path) | Self::Directory(path) => path.as_ref(),
        }
    }
}

/**
    A resolved module path for Luau, containing both the source module path,
    and the corresponding target path on the current filesystem.

    See [`LuauFilePath`] and [`LuauModulePath::resolve`] for more information.
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuauModulePath {
    // The originating module path
    source: PathBuf,
    // The target filesystem path
    target: LuauFilePath,
}

impl LuauModulePath {
    /**
        Resolves an existing file or directory path for the given *module* path.

        Given a *module* path "path/to/module", these files will be searched:

        - `path/to/module.luau`
        - `path/to/module.lua`
        - `path/to/module/init.luau`
        - `path/to/module/init.lua`

        If the given path ("path/to/module") is a directory instead,
        and it exists, it will be returned without any modifications.

        # Errors

        - If the given module path is ambiguous.
        - If the given module path does not resolve to a valid file or directory.
    */
    pub fn resolve(module: impl Into<PathBuf>) -> Result<Self, LuaNavigateError> {
        let source = module.into();
        let target = LuauFilePath::resolve(&source)?;
        Ok(Self { source, target })
    }

    /**
        Returns the source module path.
    */
    #[must_use]
    pub fn source(&self) -> &Path {
        &self.source
    }

    /**
        Returns the target file path.
    */
    #[must_use]
    pub fn target(&self) -> &LuauFilePath {
        &self.target
    }
}

impl AsRef<Path> for LuauModulePath {
    fn as_ref(&self) -> &Path {
        self.target.as_ref()
    }
}

impl fmt::Display for LuauModulePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().display().fmt(f)
    }
}
