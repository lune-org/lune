/*!
    Utilities for working with Rust standard library paths.
*/

use std::{
    env::{current_dir, current_exe},
    ffi::OsStr,
    path::{Component, MAIN_SEPARATOR, Path, PathBuf},
    sync::{Arc, LazyLock},
};

use path_clean::PathClean;

static CWD: LazyLock<Arc<Path>> = LazyLock::new(create_cwd);
static EXE: LazyLock<Arc<Path>> = LazyLock::new(create_exe);

fn create_cwd() -> Arc<Path> {
    let mut cwd = current_dir()
        .expect("failed to find current working directory")
        .to_str()
        .expect("current working directory is not valid UTF-8")
        .to_string();
    if !cwd.ends_with(MAIN_SEPARATOR) {
        cwd.push(MAIN_SEPARATOR);
    }
    dunce::canonicalize(cwd)
        .expect("failed to canonicalize current working directory")
        .into()
}

fn create_exe() -> Arc<Path> {
    let exe = current_exe()
        .expect("failed to find current executable")
        .to_str()
        .expect("current executable path is not valid UTF-8")
        .to_string();
    dunce::canonicalize(exe)
        .expect("failed to canonicalize current executable path")
        .into()
}

/**
    Gets the current working directory as an absolute path.

    This absolute path is canonicalized and does not contain any `.` or `..`
    components, and it is also in a friendly (non-UNC) format.

    This path is also guaranteed to:

    - Be valid UTF-8.
    - End with the platform's main path separator.
*/
#[must_use]
pub fn get_current_dir() -> Arc<Path> {
    Arc::clone(&CWD)
}

/**
    Gets the path to the current executable as an absolute path.

    This absolute path is canonicalized and does not contain any `.` or `..`
    components, and it is also in a friendly (non-UNC) format.

    This path is also guaranteed to:

    - Be valid UTF-8.
*/
#[must_use]
pub fn get_current_exe() -> Arc<Path> {
    Arc::clone(&EXE)
}

/**
    Cleans a path.

    See the [`path_clean`] crate for more information on what cleaning a path does.
*/
#[must_use]
pub fn clean_path(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref().clean()
}

/**
    Makes a path absolute, if it is relative, and then cleans it.

    Relative paths are resolved against the current working directory.

    See the [`path_clean`] crate for more information on what cleaning a path does.
*/
#[must_use]
pub fn clean_path_and_make_absolute(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_relative() {
        CWD.join(path).clean()
    } else {
        path.clean()
    }
}

/**
    Appends the given extension to the path.

    Does not replace or modify any existing extension(s).
*/
#[must_use]
pub fn append_extension(path: impl AsRef<Path>, ext: impl AsRef<OsStr>) -> PathBuf {
    let path = path.as_ref();
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

/**
    Normalizes the given relative path.

    This will clean the path, removing any redundant components,
    and ensure that it has a leading "current dir" (`./`) component.
*/
#[must_use]
pub fn relative_path_normalize(path: impl AsRef<Path>) -> PathBuf {
    let path = clean_path(path);

    let mut it = path.components().peekable();
    if it.peek().is_none_or(|c| matches!(c, Component::Normal(..))) {
        std::iter::once(Component::CurDir).chain(it).collect()
    } else {
        path
    }
}

/**
    Pops the relative path up to the parent directory, pushing "parent dir" (`..`)
    components to the front of the path when it no longer has any normal components.

    This means that unlike [`PathBuf::pop`], this function may be called an arbitrary
    number of times, and represent parent folders without first canonicalizing paths.
*/
pub fn relative_path_parent(rel: &mut PathBuf) {
    if rel.as_os_str() == Component::CurDir.as_os_str() {
        *rel = PathBuf::from(Component::ParentDir.as_os_str());
    } else if rel.components().all(|c| c == Component::ParentDir) {
        rel.push(Component::ParentDir.as_os_str());
    } else {
        rel.pop();
    }
}
