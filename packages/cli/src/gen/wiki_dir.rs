use std::{collections::HashMap, fmt::Write, path::PathBuf};

use anyhow::{Context, Result};

use futures_util::future::try_join_all;
use tokio::fs::{create_dir_all, write};

use super::definitions::{DefinitionsItem, DefinitionsItemKind, DefinitionsTree};

const GENERATED_COMMENT_TAG: &str = "<!-- @generated with lune-cli -->";
const CATEGORY_NONE: &str = "uncategorized";

pub async fn generate_from_type_definitions(contents: &str) -> Result<()> {
    let tree = DefinitionsTree::from_type_definitions(contents)?;
    let mut dirs_to_write = Vec::new();
    let mut files_to_write = Vec::new();
    // Create the wiki dir at the repo root
    let path_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .unwrap();
    let path_wiki_dir = path_root.join("wiki");
    dirs_to_write.push(path_wiki_dir.clone());
    // Sort doc items into subcategories based on globals
    let mut api_reference: HashMap<&str, Vec<DefinitionsItem>> = HashMap::new();
    for top_level_item in tree
        .children()
        .iter()
        .filter(|top_level| top_level.is_exported())
    {
        match top_level_item.kind() {
            DefinitionsItemKind::Table => {
                let category_name = top_level_item
                    .get_name()
                    .context("Missing name for top-level doc item")?;
                let category = match api_reference.contains_key(category_name) {
                    true => api_reference.get_mut(category_name).unwrap(),
                    false => {
                        api_reference.insert(category_name, vec![]);
                        api_reference.get_mut(category_name).unwrap()
                    }
                };
                category.push(top_level_item.clone());
            }
            DefinitionsItemKind::Function => {
                let category = match api_reference.contains_key(CATEGORY_NONE) {
                    true => api_reference.get_mut(CATEGORY_NONE).unwrap(),
                    false => {
                        api_reference.insert(CATEGORY_NONE, vec![]);
                        api_reference.get_mut(CATEGORY_NONE).unwrap()
                    }
                };
                category.push(top_level_item.clone());
            }
            _ => unimplemented!("Globals other than tables and functions are not yet implemented"),
        }
    }
    // Generate our api reference folder
    let path_api_ref = path_wiki_dir.join("api-reference");
    dirs_to_write.push(path_api_ref.clone());
    // Generate files for all subcategories
    for (category_name, category_items) in api_reference {
        if category_items.len() == 1 {
            let item = category_items.first().unwrap();
            let path = path_api_ref.join(category_name).with_extension("md");
            let mut contents = String::new();
            write!(contents, "{GENERATED_COMMENT_TAG}\n\n")?;
            generate_markdown_documentation(&mut contents, item)?;
            files_to_write.push((path, post_process_docs(contents)));
        } else {
            let path_subcategory = path_api_ref.join(category_name);
            dirs_to_write.push(path_subcategory.clone());
            for item in category_items {
                let item_name = item
                    .get_name()
                    .context("Missing name for subcategory doc item")?;
                let path = path_subcategory.join(item_name).with_extension("md");
                let mut contents = String::new();
                write!(contents, "{GENERATED_COMMENT_TAG}\n\n")?;
                generate_markdown_documentation(&mut contents, &item)?;
                files_to_write.push((path, post_process_docs(contents)));
            }
        }
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

fn generate_markdown_documentation(contents: &mut String, item: &DefinitionsItem) -> Result<()> {
    match item.kind() {
        DefinitionsItemKind::Table => {
            write!(
                contents,
                "\n# {}\n",
                item.get_name().context("Table is missing a name")?
            )?;
        }
        DefinitionsItemKind::Property => {
            write!(
                contents,
                "\n### `{}`\n",
                item.get_name().context("Property is missing a name")?
            )?;
        }
        DefinitionsItemKind::Function => {
            write!(
                contents,
                "\n### `{}`\n",
                item.get_name().context("Function is missing a name")?
            )?;
        }
        DefinitionsItemKind::Description => {
            write!(
                contents,
                "\n{}\n",
                item.get_value().context("Description is missing a value")?
            )?;
        }
        _ => {}
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
    for description in descriptions {
        generate_markdown_documentation(contents, description)?;
    }
    if !properties.is_empty() {
        write!(contents, "\n\n---\n\n## Properties\n\n")?;
    }
    for property in properties {
        generate_markdown_documentation(contents, property)?;
    }
    if !functions.is_empty() {
        write!(contents, "\n\n---\n\n## Functions\n\n")?;
    }
    for function in functions {
        generate_markdown_documentation(contents, function)?;
    }
    Ok(())
}

fn post_process_docs(contents: String) -> String {
    contents.replace("\n\n\n", "\n\n")
}
