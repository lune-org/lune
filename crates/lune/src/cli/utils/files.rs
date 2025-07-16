use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use console::style;
use directories::UserDirs;

use lune_utils::path::{LuauFilePath, LuauModulePath, get_current_dir};

const LUNE_COMMENT_PREFIX: &str = "-->";

/**
    Discovers a script file path based on a given module path *or* file path.

    See the documentation for [`LuauModulePath`] for more information about
    what a module path vs a script path is.
*/
pub fn discover_script_path(path: impl Into<PathBuf>, in_home_dir: bool) -> Result<PathBuf> {
    // First, for legacy compatibility, we will strip any lua/luau file extension,
    // and if the entire file stem is simply "init", we will get rid of that too
    // This lets users pass "dir/init.luau" and have it resolve to simply "dir",
    // which is a valid luau module path, while "dir/init.luau" is not
    let path = LuauModulePath::strip(path);

    // If we got an absolute path, we should not modify it,
    // otherwise we should either resolve against home or cwd
    let path = if path.is_absolute() {
        path
    } else if in_home_dir {
        UserDirs::new()
            .context("Missing home directory")?
            .home_dir()
            .join(path)
    } else {
        get_current_dir().join(path)
    };

    // The rest of the logic should follow Luau module path resolution rules
    match LuauModulePath::resolve(&path) {
        Err(e) => Err(anyhow!(
            "Failed to resolve script at path {} ({})",
            style(path.display()).yellow(),
            style(format!("{e:?}")).red()
        )),
        Ok(m) => match m.target() {
            LuauFilePath::File(f) => Ok(f.clone()),
            LuauFilePath::Directory(_) => Err(anyhow!(
                "Failed to resolve script at path {}\
                \nThe path is a directory without an init file",
                style(path.display()).yellow()
            )),
        },
    }
}

/**
    Discovers a script file path based on a given script name, and tries to
    find scripts in `lune` and `.lune` folders if one was not directly found.

    Note that looking in `lune` and `.lune` folders is automatically
    disabled if the given script name is an absolute path.

    Behavior is otherwise exactly the same as for `discover_script_file_path`.
*/
pub fn discover_script_path_including_lune_dirs(path: impl AsRef<Path>) -> Result<PathBuf> {
    let path: &Path = path.as_ref();
    match discover_script_path(path, false) {
        Ok(path) => Ok(path),
        Err(e) => {
            // If we got any absolute path it means the user has also
            // told us to not look in any special relative directories
            // so we should error right away with the first err message
            if path.is_absolute() {
                return Err(e);
            }

            // Otherwise we take a look in relative lune and .lune
            // directories + the home directory for the current user
            let res = discover_script_path(Path::new("lune").join(path), false)
                .or_else(|_| discover_script_path(Path::new(".lune").join(path), false))
                .or_else(|_| discover_script_path(Path::new("lune").join(path), true))
                .or_else(|_| discover_script_path(Path::new(".lune").join(path), true));

            match res {
                // NOTE: The first error message is generally more
                // descriptive than the ones for the lune subfolders
                Err(_) => Err(e),
                Ok(path) => Ok(path),
            }
        }
    }
}

pub fn parse_lune_description_from_file(contents: &str) -> Option<String> {
    let mut comment_lines = Vec::new();
    for line in contents.lines() {
        if let Some(stripped) = line.strip_prefix(LUNE_COMMENT_PREFIX) {
            comment_lines.push(stripped);
        } else {
            break;
        }
    }
    if comment_lines.is_empty() {
        None
    } else {
        let shortest_indent = comment_lines.iter().fold(usize::MAX, |acc, line| {
            let first_alphanumeric = line.find(char::is_alphanumeric).unwrap();
            acc.min(first_alphanumeric)
        });
        let unindented_lines = comment_lines
            .iter()
            .map(|line| line[shortest_indent..].to_string())
            .collect::<Vec<_>>()
            .join(" ");
        Some(unindented_lines)
    }
}
