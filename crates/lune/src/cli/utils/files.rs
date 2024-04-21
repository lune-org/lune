use std::{
    fs::Metadata,
    path::{PathBuf, MAIN_SEPARATOR},
};

use anyhow::{anyhow, bail, Result};
use console::style;
use directories::UserDirs;
use itertools::Itertools;
use once_cell::sync::Lazy;

const LUNE_COMMENT_PREFIX: &str = "-->";

static ERR_MESSAGE_HELP_NOTE: Lazy<String> = Lazy::new(|| {
    format!(
        "To run this file, either:\n{}\n{}",
        format_args!(
            "{} rename it to use a {} or {} extension",
            style("-").dim(),
            style(".luau").blue(),
            style(".lua").blue()
        ),
        format_args!(
            "{} pass it as an absolute path instead of relative",
            style("-").dim()
        ),
    )
});

/**
    Discovers a script file path based on a given script name.

    Script discovery is done in several steps here for the best possible user experience:

    1. If we got a file that definitely exists, make sure it is either
        - using an absolute path
        - has the lua or luau extension
    2. If we got a directory, check if it has an `init` file to use, and if it doesn't, let the user know
    3. If we got an absolute path, don't check any extensions, just let the user know it didn't exist
    4. If we got a relative path with no extension, also look for a file with a lua or luau extension
    5. No other options left, the file simply did not exist

    This behavior ensures that users can do pretty much whatever they want if they pass in an absolute
    path, and that they then have control over script discovery behavior, whereas if they pass in
    a relative path we will instead try to be as permissive as possible for user-friendliness
*/
pub fn discover_script_path(path: impl AsRef<str>, in_home_dir: bool) -> Result<PathBuf> {
    // NOTE: We don't actually support any platforms without home directories,
    // but just in case the user has some strange configuration and it cannot
    // be found we should at least throw a nice error instead of panicking
    let path = path.as_ref();
    let file_path = if in_home_dir {
        match UserDirs::new() {
            Some(dirs) => dirs.home_dir().join(path),
            None => {
                bail!(
                    "No file was found at {}\nThe home directory does not exist",
                    style(path).yellow()
                )
            }
        }
    } else {
        PathBuf::from(path)
    };
    // NOTE: We use metadata directly here to try to
    // avoid accessing the file path more than once
    let file_meta = file_path.metadata();
    let is_file = file_meta.as_ref().map_or(false, Metadata::is_file);
    let is_dir = file_meta.as_ref().map_or(false, Metadata::is_dir);
    let is_abs = file_path.is_absolute();
    let ext = file_path.extension();
    if is_file {
        if is_abs {
            Ok(file_path)
        } else if let Some(ext) = file_path.extension() {
            match ext {
                e if e == "lua" || e == "luau" => Ok(file_path),
                _ => Err(anyhow!(
                    "A file was found at {} but it uses the '{}' file extension\n{}",
                    style(file_path.display()).green(),
                    style(ext.to_string_lossy()).blue(),
                    *ERR_MESSAGE_HELP_NOTE
                )),
            }
        } else {
            Err(anyhow!(
                "A file was found at {} but it has no file extension\n{}",
                style(file_path.display()).green(),
                *ERR_MESSAGE_HELP_NOTE
            ))
        }
    } else if is_dir {
        match (
            discover_script_path(format!("{path}/init.luau"), in_home_dir),
            discover_script_path(format!("{path}/init.lua"), in_home_dir),
        ) {
            (Ok(path), _) | (_, Ok(path)) => Ok(path),
            _ => Err(anyhow!(
                "No file was found at {}, found a directory without an init file",
                style(file_path.display()).yellow()
            )),
        }
    } else if is_abs && !in_home_dir {
        Err(anyhow!(
            "No file was found at {}",
            style(file_path.display()).yellow()
        ))
    } else if ext.is_none() {
        let file_path_lua = file_path.with_extension("lua");
        let file_path_luau = file_path.with_extension("luau");
        if file_path_lua.is_file() {
            Ok(file_path_lua)
        } else if file_path_luau.is_file() {
            Ok(file_path_luau)
        } else {
            Err(anyhow!(
                "No file was found at {}",
                style(file_path.display()).yellow()
            ))
        }
    } else {
        Err(anyhow!(
            "No file was found at {}",
            style(file_path.display()).yellow()
        ))
    }
}

/**
    Discovers a script file path based on a given script name, and tries to
    find scripts in `lune` and `.lune` folders if one was not directly found.

    Note that looking in `lune` and `.lune` folders is automatically
    disabled if the given script name is an absolute path.

    Behavior is otherwise exactly the same as for `discover_script_file_path`.
*/
pub fn discover_script_path_including_lune_dirs(path: &str) -> Result<PathBuf> {
    match discover_script_path(path, false) {
        Ok(path) => Ok(path),
        Err(e) => {
            // If we got any absolute path it means the user has also
            // told us to not look in any special relative directories
            // so we should error right away with the first err message
            if PathBuf::from(path).is_absolute() {
                return Err(e);
            }
            // Otherwise we take a look in relative lune and .lune
            // directories + the home directory for the current user
            let res = discover_script_path(format!("lune{MAIN_SEPARATOR}{path}"), false)
                .or_else(|_| discover_script_path(format!(".lune{MAIN_SEPARATOR}{path}"), false))
                .or_else(|_| discover_script_path(format!("lune{MAIN_SEPARATOR}{path}"), true))
                .or_else(|_| discover_script_path(format!(".lune{MAIN_SEPARATOR}{path}"), true));

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
            .map(|line| &line[shortest_indent..])
            // Replace newlines with a single space inbetween instead
            .interleave(std::iter::repeat(" ").take(comment_lines.len() - 1))
            .collect();
        Some(unindented_lines)
    }
}

pub fn strip_shebang(mut contents: Vec<u8>) -> Vec<u8> {
    if contents.starts_with(b"#!") {
        if let Some(first_newline_idx) =
            contents
                .iter()
                .enumerate()
                .find_map(|(idx, c)| if *c == b'\n' { Some(idx) } else { None })
        {
            // NOTE: We keep the newline here on purpose to preserve
            // correct line numbers in stack traces, the only reason
            // we strip the shebang is to get the lua script to parse
            // and the extra newline is not really a problem for that
            contents.drain(..first_newline_idx);
        }
    }
    contents
}
