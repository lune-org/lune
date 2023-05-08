use std::{collections::HashMap, fmt::Write};

use anyhow::{Context, Result};
use directories::UserDirs;

use futures_util::future::try_join_all;
use tokio::fs::{create_dir_all, write};

use super::definitions::{DefinitionsItem, DefinitionsTree};

const GENERATED_COMMENT_TAG: &str = "--!strict";

#[allow(clippy::too_many_lines)]
pub async fn generate_from_type_definitions(
    api_reference: HashMap<String, DefinitionsTree>,
) -> Result<()> {
    let mut dirs_to_write = Vec::new();
    let mut files_to_write = Vec::new();
    // Create the typedefs dir in the users cache dir
    let cache_dir = UserDirs::new()
        .context("Failed to find user home directory")?
        .home_dir()
        .join(".lune")
        .join("typedefs")
        .join(env!("CARGO_PKG_VERSION"));
    dirs_to_write.push(cache_dir.clone());
    // Make typedef files
    for (category_name, category_tree) in api_reference {
        let path = cache_dir
            .join(category_name.to_ascii_lowercase())
            .with_extension("luau");
        let mut contents = String::new();
        write!(
            contents,
            "{GENERATED_COMMENT_TAG}\n-- @lune/{} {}\n",
            category_name.to_lowercase(),
            env!("CARGO_PKG_VERSION")
        )?;
        write_tree(&mut contents, category_name, category_tree)?;
        files_to_write.push((path, contents));
    }
    // Write all dirs and files only when we know generation was successful
    let futs_dirs = dirs_to_write
        .drain(..)
        .map(create_dir_all)
        .collect::<Vec<_>>();
    let futs_files = files_to_write
        .drain(..)
        .map(|(path, contents)| write(path, contents))
        .collect::<Vec<_>>();
    try_join_all(futs_dirs).await?;
    try_join_all(futs_files).await?;
    Ok(())
}

fn make_return_table_item(item: &DefinitionsItem) -> Result<String> {
    let mut description = String::new();
    if let Some(desc) = item.children().iter().find(|child| child.is_description()) {
        write!(description, "\n{}\n", desc.get_value().unwrap().trim())?;
        for tag in item.children().iter().filter(|child| child.is_tag()) {
            let tag_name = tag.get_name().unwrap();
            if tag_name == "param" {
                write!(
                    description,
                    "\n@param {} {}",
                    tag.get_meta().unwrap(),
                    tag.get_value().unwrap()
                )?;
            } else if tag_name == "return" {
                write!(description, "\n@return {}", tag.get_value().unwrap())?;
            }
        }
    }

    let mut contents = String::new();
    if item.is_function() {
        let args = item
            .args()
            .iter()
            .map(|arg| format!("{}: {}", arg.name.trim(), arg.typedef.trim()))
            .collect::<Vec<_>>()
            .join(", ");
        write!(contents, "function ({args})")?;
        write!(contents, "\n\treturn nil :: any")?;
        write!(contents, "\nend,")?;
    } else if item.is_property() {
        write!(contents, "(nil :: any) :: {},", item.get_type().unwrap())?;
    }

    Ok(format!(
        "\n--[=[{}\n]=]\n{} = {}",
        description.trim_end().replace('\n', "\n\t"),
        item.get_name().unwrap_or("_"),
        contents
    ))
}

fn write_tree(contents: &mut String, name: String, root: DefinitionsTree) -> Result<()> {
    let main = root
        .children()
        .iter()
        .find(|c| matches!(c.get_name(), Some(s) if s.to_lowercase() == name.to_lowercase()))
        .expect("Failed to find main export for generating typedef file");

    let mut description = String::new();
    if let Some(desc) = main.children().iter().find(|child| child.is_description()) {
        write!(description, "\n{}\n", desc.get_value().unwrap().trim())?;
    }

    let children = root
        .children()
        .iter()
        .filter(|child| child != &main)
        .collect::<Vec<_>>();
    for child in children {
        if child.is_type() || child.is_table() || child.is_function() || child.is_property() {
            let mut child_description = String::new();
            if let Some(desc) = child.children().iter().find(|child| child.is_description()) {
                write!(
                    child_description,
                    "\n{}\n",
                    desc.get_value().unwrap().trim()
                )?;
                write!(
                    contents,
                    "\n--[=[{}\n]=]",
                    child_description.trim_end().replace('\n', "\n\t"),
                )?;
            }
            if child.is_exported() {
                write!(contents, "\nexport ")?;
            }
            writeln!(
                contents,
                "type {} = {}",
                child.get_name().unwrap(),
                child.get_type().unwrap()
            )?;
        }
    }

    let mut ret_table = String::new();
    for child in main
        .children()
        .iter()
        .filter(|child| child.is_function() || child.is_property())
    {
        write!(ret_table, "{}", make_return_table_item(child)?)?;
    }

    write!(
        contents,
        "\n--[=[{}\n]=]\nreturn {{\n{}\n}}\n",
        description.trim_end().replace('\n', "\n\t"),
        ret_table.trim_end().replace('\n', "\n\t")
    )?;

    Ok(())
}
