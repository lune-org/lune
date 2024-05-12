#![allow(clippy::match_same_arms)]

use std::{cmp::Ordering, ffi::OsStr, fmt::Write as _, path::PathBuf};

use anyhow::{bail, Result};
use console::Style;
use directories::UserDirs;
use once_cell::sync::Lazy;
use tokio::{fs, io};

use super::files::{discover_script_path, parse_lune_description_from_file};

pub static COLOR_BLUE: Lazy<Style> = Lazy::new(|| Style::new().blue());
pub static STYLE_DIM: Lazy<Style> = Lazy::new(|| Style::new().dim());

pub async fn find_lune_scripts(in_home_dir: bool) -> Result<Vec<(String, String)>> {
    let base_path = if in_home_dir {
        UserDirs::new().unwrap().home_dir().to_path_buf()
    } else {
        PathBuf::new()
    };
    let mut lune_dir = fs::read_dir(base_path.join("lune")).await;
    if lune_dir.is_err() {
        lune_dir = fs::read_dir(base_path.join(".lune")).await;
    }
    match lune_dir {
        Ok(mut dir) => {
            let mut files = Vec::new();
            while let Some(entry) = dir.next_entry().await? {
                let meta = entry.metadata().await?;
                if meta.is_file() {
                    let contents = fs::read(entry.path()).await?;
                    files.push((entry, meta, contents));
                } else if meta.is_dir() {
                    let entry_path_os = entry.path();

                    let Some(dir_path) = entry_path_os.to_str() else {
                        bail!("Failed to cast path to string slice.")
                    };

                    let Ok(init_file_path) = discover_script_path(dir_path, in_home_dir) else {
                        continue;
                    };

                    let contents = fs::read(init_file_path).await?;

                    files.push((entry, meta, contents));
                }
            }
            let parsed: Vec<_> = files
                .iter()
                .filter(|(entry, _, _)| {
                    let mut is_match = matches!(
                        entry.path().extension().and_then(OsStr::to_str),
                        Some("lua" | "luau")
                    );

                    // If the entry is not a lua or luau file, and is a directory,
                    // then we check if it contains a init.lua(u), and if it does,
                    // we include it in our Vec
                    if !is_match
                        && entry.path().is_dir()
                        && (entry.path().join("init.lua").exists()
                            || entry.path().join("init.luau").exists())
                    {
                        is_match = true;
                    }

                    is_match
                })
                .map(|(entry, _, contents)| {
                    let contents_str = String::from_utf8_lossy(contents);
                    let file_path = entry.path().with_extension("");
                    let file_name = file_path.file_name().unwrap().to_string_lossy();
                    let description = parse_lune_description_from_file(&contents_str);
                    (file_name.to_string(), description.unwrap_or_default())
                })
                .collect();
            Ok(parsed)
        }
        Err(e) if matches!(e.kind(), io::ErrorKind::NotFound) => {
            bail!("No lune directory was found.")
        }
        Err(e) => {
            bail!("Failed to read lune files!\n{e}")
        }
    }
}

pub fn sort_lune_scripts(scripts: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut sorted = scripts;
    sorted.sort_by(|left, right| {
        // Prefer scripts that have a description
        let left_has_desc = !left.1.is_empty();
        let right_has_desc = !right.1.is_empty();
        if left_has_desc == right_has_desc {
            // If both have a description or both
            // have no description, we sort by name
            left.0.cmp(&right.0)
        } else if left_has_desc {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    sorted
}

pub fn write_lune_scripts_list(buffer: &mut String, scripts: Vec<(String, String)>) -> Result<()> {
    let longest_file_name_len = scripts
        .iter()
        .fold(0, |acc, (file_name, _)| acc.max(file_name.len()));
    let script_with_description_exists = scripts.iter().any(|(_, desc)| !desc.is_empty());
    // Pre-calculate some strings that will be used often
    let prefix = format!("{}  ", COLOR_BLUE.apply_to('>'));
    let separator = format!("{}", STYLE_DIM.apply_to('-'));
    // Write the entire output to a buffer, doing this instead of using individual
    // writeln! calls will ensure that no output get mixed up in between these lines
    if script_with_description_exists {
        for (file_name, description) in scripts {
            if description.is_empty() {
                write!(buffer, "\n{prefix}{file_name}")?;
            } else {
                let mut lines = description.lines();
                let first_line = lines.next().unwrap_or_default();
                let file_spacing = " ".repeat(file_name.len());
                let line_spacing = " ".repeat(longest_file_name_len - file_name.len());
                write!(
                    buffer,
                    "\n{prefix}{file_name}{line_spacing}  {separator} {}",
                    COLOR_BLUE.apply_to(first_line)
                )?;
                for line in lines {
                    write!(
                        buffer,
                        "\n{prefix}{file_spacing}{line_spacing}    {}",
                        COLOR_BLUE.apply_to(line)
                    )?;
                }
            }
        }
    } else {
        for (file_name, _) in scripts {
            write!(buffer, "\n{prefix}{file_name}")?;
        }
    }
    // Finally, write an ending newline
    writeln!(buffer)?;
    Ok(())
}
