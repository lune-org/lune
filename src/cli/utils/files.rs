use std::path::{PathBuf, MAIN_SEPARATOR};

use anyhow::{bail, Result};

const LUNE_COMMENT_PREFIX: &str = "-->";

pub fn find_luau_file_path(path: &str) -> Option<PathBuf> {
    let file_path = PathBuf::from(path);
    if let Some(ext) = file_path.extension() {
        match ext {
            e if e == "lua" || e == "luau" && file_path.exists() => Some(file_path),
            _ => None,
        }
    } else {
        let file_path_lua = PathBuf::from(path).with_extension("lua");
        if file_path_lua.exists() {
            Some(file_path_lua)
        } else {
            let file_path_luau = PathBuf::from(path).with_extension("luau");
            if file_path_luau.exists() {
                Some(file_path_luau)
            } else {
                None
            }
        }
    }
}

pub fn find_parse_file_path(path: &str) -> Result<PathBuf> {
    let parsed_file_path = find_luau_file_path(path)
        .or_else(|| find_luau_file_path(&format!("lune{MAIN_SEPARATOR}{path}")))
        .or_else(|| find_luau_file_path(&format!(".lune{MAIN_SEPARATOR}{path}")));
    if let Some(file_path) = parsed_file_path {
        if file_path.exists() {
            Ok(file_path)
        } else {
            bail!("File does not exist at path: '{}'", path)
        }
    } else {
        bail!("Invalid file path: '{}'", path)
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
            .collect();
        Some(unindented_lines)
    }
}
