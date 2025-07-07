use std::{
    env::{current_dir, current_exe},
    path::{Path, PathBuf, MAIN_SEPARATOR},
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
pub fn clean_path(path: impl AsRef<Path>) -> PathBuf {
    path.as_ref().clean()
}

/**
    Makes a path absolute, if it is relative, and then cleans it.

    Relative paths are resolved against the current working directory.

    See the [`path_clean`] crate for more information on what cleaning a path does.
*/
pub fn clean_path_and_make_absolute(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_relative() {
        CWD.join(path).clean()
    } else {
        path.clean()
    }
}
