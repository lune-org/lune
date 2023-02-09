use std::{cmp::Ordering, fmt::Write as _};

use anyhow::{bail, Result};
use console::Style;
use lazy_static::lazy_static;
use tokio::{fs, io};

use super::files::parse_lune_description_from_file;

lazy_static! {
    pub static ref COLOR_BLUE: Style = Style::new().blue();
    pub static ref STYLE_DIM: Style = Style::new().dim();
}

pub async fn find_lune_scripts() -> Result<Vec<(String, String)>> {
    let mut lune_dir = fs::read_dir("lune").await;
    if lune_dir.is_err() {
        lune_dir = fs::read_dir(".lune").await;
    }
    match lune_dir {
        Ok(mut dir) => {
            let mut files = Vec::new();
            while let Some(entry) = dir.next_entry().await? {
                let meta = entry.metadata().await?;
                if meta.is_file() {
                    let contents = fs::read_to_string(entry.path()).await?;
                    files.push((entry, meta, contents));
                }
            }
            let parsed: Vec<_> = files
                .iter()
                .map(|(entry, _, contents)| {
                    let file_path = entry.path().with_extension("");
                    let file_name = file_path.file_name().unwrap().to_string_lossy();
                    let description = parse_lune_description_from_file(contents);
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

pub fn print_lune_scripts(scripts: Vec<(String, String)>) -> Result<()> {
    let longest_file_name_len = scripts
        .iter()
        .fold(0, |acc, (file_name, _)| acc.max(file_name.len()));
    let script_with_description_exists = scripts.iter().any(|(_, desc)| !desc.is_empty());
    // Pre-calculate some strings that will be used often
    let prefix = format!("{}  ", COLOR_BLUE.apply_to('>'));
    let separator = format!("{}", STYLE_DIM.apply_to('-'));
    // Write the entire output to a buffer, doing this instead of using individual
    // println! calls will ensure that no output get mixed up in between these lines
    let mut buffer = String::new();
    if script_with_description_exists {
        for (file_name, description) in scripts {
            if description.is_empty() {
                write!(&mut buffer, "\n{prefix}{file_name}")?;
            } else {
                let mut lines = description.lines();
                let first_line = lines.next().unwrap_or_default();
                let file_spacing = " ".repeat(file_name.len());
                let line_spacing = " ".repeat(longest_file_name_len - file_name.len());
                write!(
                    &mut buffer,
                    "\n{prefix}{file_name}{line_spacing}  {separator} {}",
                    COLOR_BLUE.apply_to(first_line)
                )?;
                for line in lines {
                    write!(
                        &mut buffer,
                        "\n{prefix}{file_spacing}{line_spacing}    {}",
                        COLOR_BLUE.apply_to(line)
                    )?;
                }
            }
        }
    } else {
        for (file_name, _) in scripts {
            write!(&mut buffer, "\n{prefix}{file_name}")?;
        }
    }
    // Finally, print the entire buffer out
    // with an ending newline added to it
    println!("{buffer}");
    Ok(())
}
