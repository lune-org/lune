use std::{collections::HashMap, fmt::Write, path::PathBuf};

use anyhow::{Context, Result};

use futures_util::future::try_join_all;
use tokio::fs::{create_dir_all, write};

use super::definitions::{
    DefinitionsItem, DefinitionsItemBuilder, DefinitionsItemKind, DefinitionsItemTag,
    DefinitionsTree,
};

const GENERATED_COMMENT_TAG: &str = "<!-- @generated with lune-cli -->";

#[allow(clippy::too_many_lines)]
pub async fn generate_from_type_definitions(
    definitions: HashMap<String, DefinitionsTree>,
) -> Result<()> {
    let mut dirs_to_write = Vec::new();
    let mut files_to_write = Vec::new();
    // Create the gitbook dir at the repo root
    let path_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .unwrap();
    let path_gitbook_dir = path_root.join("gitbook");
    let path_gitbook_docs_dir = path_gitbook_dir.join("docs");
    let path_gitbook_pages_dir = path_gitbook_docs_dir.join("pages");
    let path_gitbook_api_dir = path_gitbook_pages_dir.join("api");
    dirs_to_write.push(path_gitbook_dir.clone());
    dirs_to_write.push(path_gitbook_docs_dir.clone());
    dirs_to_write.push(path_gitbook_pages_dir.clone());
    dirs_to_write.push(path_gitbook_api_dir.clone());

    // Convert definition trees into single root items so that we can parse and write markdown recursively
    let mut typedef_items = HashMap::new();
    for (typedef_name, typedef_contents) in definitions {
        let main = typedef_contents
        .children()
        .iter()
        .find(
            |c| matches!(c.get_name(), Some(s) if s.to_lowercase() == typedef_name.to_lowercase()),
        )
        .expect("Failed to find main export for generating typedef file");

        let children = typedef_contents
            .children()
            .iter()
            .filter_map(|child| {
                if child == main {
                    None
                } else {
                    Some(
                        DefinitionsItemBuilder::from(child)
                            .with_kind(DefinitionsItemKind::Type)
                            .build()
                            .unwrap(),
                    )
                }
            })
            .collect::<Vec<_>>();

        let root = DefinitionsItemBuilder::new()
            .with_kind(main.kind())
            .with_name(main.get_name().unwrap())
            .with_children(main.children())
            .with_children(&children);
        let root_item = root.build().expect("Failed to build root definitions item");

        typedef_items.insert(typedef_name.to_string(), root_item);
    }

    // Generate files for all subcategories
    for (category_name, category_item) in typedef_items {
        let path = path_gitbook_api_dir
            .join(category_name.to_ascii_lowercase())
            .with_extension("md");
        let mut contents = String::new();
        write!(contents, "{GENERATED_COMMENT_TAG}\n\n")?;
        generate_markdown_documentation(&mut contents, &category_item, None, 0)?;
        files_to_write.push((path, post_process_docs(contents)));
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

fn get_name(item: &DefinitionsItem) -> Result<String> {
    item.children()
        .iter()
        .find_map(|child| {
            if child.is_tag() {
                if let Ok(DefinitionsItemTag::Class(c)) = DefinitionsItemTag::try_from(child) {
                    Some(c)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .or_else(|| item.get_name().map(ToString::to_string))
        .context("Definitions item is missing a name")
}

#[allow(clippy::too_many_lines)]
fn generate_markdown_documentation(
    contents: &mut String,
    item: &DefinitionsItem,
    parent: Option<&DefinitionsItem>,
    depth: usize,
) -> Result<()> {
    match item.kind() {
        DefinitionsItemKind::Type
        | DefinitionsItemKind::Table
        | DefinitionsItemKind::Property
        | DefinitionsItemKind::Function => {
            write!(
                contents,
                "\n{} {}\n",
                if item.is_table() { "#" } else { "###" },
                get_name(item)?
            )?;
        }
        DefinitionsItemKind::Description => {
            let desc = item.get_value().context("Description is missing a value")?;
            write!(
                contents,
                "\n{}\n",
                if depth >= 2 {
                    // HACK: We know our typedefs are formatted like this and
                    // it looks nicer to have this bolding instead of two
                    // headers using "###" in the function definition
                    desc.replace("### Example usage", "**Example usage:**")
                } else {
                    desc.to_string()
                }
            )?;
        }
        _ => {}
    }
    if item.is_function() && !item.args().is_empty() {
        let args = item
            .args()
            .iter()
            .map(|arg| format!("{}: {}", arg.name.trim(), arg.typedef.trim()))
            .collect::<Vec<_>>()
            .join(", ")
            .replace("_: T...", "T...");
        let func_name = item.get_name().unwrap_or("_");
        let parent_name = parent.unwrap().get_name().unwrap_or("_");
        let parent_pre = if parent_name.to_lowercase() == "uncategorized" {
            String::new()
        } else {
            format!("{parent_name}.")
        };
        write!(
            contents,
            "\n```lua\nfunction {parent_pre}{func_name}({args})\n```\n",
        )?;
    } else if item.is_type() {
        write!(
            contents,
            "\n```lua\ntype {} = {}\n```\n",
            item.get_name().unwrap_or("_"),
            item.get_type().unwrap_or_else(|| "{}".to_string()).trim()
        )?;
    }
    let descriptions = item
        .children()
        .iter()
        .filter(|child| child.is_description())
        .collect::<Vec<_>>();
    let properties = item
        .children()
        .iter()
        .filter(|child| child.is_property())
        .collect::<Vec<_>>();
    let functions = item
        .children()
        .iter()
        .filter(|child| child.is_function())
        .collect::<Vec<_>>();
    let types = item
        .children()
        .iter()
        .filter(|child| child.is_type())
        .collect::<Vec<_>>();
    for description in descriptions {
        generate_markdown_documentation(contents, description, Some(item), depth + 1)?;
    }
    if !item.is_type() {
        if !properties.is_empty() {
            write!(contents, "\n\n---\n\n## Properties\n\n")?;
        }
        for property in properties {
            generate_markdown_documentation(contents, property, Some(item), depth + 1)?;
        }
        if !functions.is_empty() {
            write!(contents, "\n\n---\n\n## Functions\n\n")?;
        }
        for function in functions {
            generate_markdown_documentation(contents, function, Some(item), depth + 1)?;
        }
        if !types.is_empty() {
            write!(contents, "\n\n---\n\n## Types\n\n")?;
        }
        for typ in types {
            generate_markdown_documentation(contents, typ, Some(item), depth + 1)?;
        }
    }
    Ok(())
}

fn post_process_docs(contents: String) -> String {
    let no_empty_lines = contents
        .lines()
        .map(|line| {
            if line.chars().all(char::is_whitespace) {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    no_empty_lines
        .replace("\n\n---", "\n---")
        .replace("\n\n\n", "\n\n")
        .replace("\n\n\n", "\n\n")
}
