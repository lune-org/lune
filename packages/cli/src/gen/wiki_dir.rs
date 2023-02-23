use std::{collections::HashMap, fmt::Write, path::PathBuf};

use anyhow::{Context, Result};

use futures_util::future::try_join_all;
use tokio::fs::{create_dir_all, write};

use super::definitions::{
    DefinitionsItem, DefinitionsItemBuilder, DefinitionsItemKind, DefinitionsItemTag,
    DefinitionsTree,
};

const GENERATED_COMMENT_TAG: &str = "<!-- @generated with lune-cli -->";
const CATEGORY_NONE_NAME: &str = "Uncategorized";
const CATEGORY_NONE_DESC: &str = "
All globals that are not available under a specific scope.

These are to be used directly without indexing a global table first.
";

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
    let mut api_reference = HashMap::new();
    let mut no_category = Vec::new();
    for top_level_item in tree
        .children()
        .iter()
        .filter(|top_level| top_level.is_exported())
    {
        match top_level_item.kind() {
            DefinitionsItemKind::Table => {
                let category_name =
                    get_name(top_level_item).context("Missing name for top-level doc item")?;
                api_reference.insert(category_name, top_level_item.clone());
            }
            DefinitionsItemKind::Function => {
                no_category.push(top_level_item.clone());
            }
            _ => unimplemented!("Globals other than tables and functions are not yet implemented"),
        }
    }
    // Insert globals with no category into a new "Uncategorized" global
    api_reference.insert(
        CATEGORY_NONE_NAME.to_string(),
        DefinitionsItemBuilder::new()
            .with_kind(DefinitionsItemKind::Table)
            .with_name("Uncategorized")
            .with_children(&no_category)
            .with_child(
                DefinitionsItemBuilder::new()
                    .with_kind(DefinitionsItemKind::Description)
                    .with_value(CATEGORY_NONE_DESC)
                    .build()?,
            )
            .build()
            .unwrap(),
    );
    // Generate files for all subcategories
    for (category_name, category_item) in api_reference {
        let path = path_wiki_dir
            .join(format!("API Reference - {category_name}"))
            .with_extension("md");
        let mut contents = String::new();
        write!(contents, "{GENERATED_COMMENT_TAG}\n\n")?;
        generate_markdown_documentation(&mut contents, &category_item)?;
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

fn generate_markdown_documentation(contents: &mut String, item: &DefinitionsItem) -> Result<()> {
    match item.kind() {
        DefinitionsItemKind::Table
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
