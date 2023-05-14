use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use include_dir::Dir;

use self::definitions::DefinitionsTree;

mod gitbook_dir;
mod typedef_files;

pub mod definitions;

pub async fn generate_gitbook_dir_from_definitions(dir: &Dir<'_>) -> Result<()> {
    let definitions = read_typedefs_dir(dir)?;
    gitbook_dir::generate_from_type_definitions(definitions).await
}

pub async fn generate_typedef_files_from_definitions(
    dir: &Dir<'_>,
) -> Result<HashMap<String, PathBuf>> {
    let contents = read_typedefs_dir_contents(dir);
    typedef_files::generate_from_type_definitions(contents).await
}

fn read_typedefs_dir_contents(dir: &Dir<'_>) -> HashMap<String, Vec<u8>> {
    let mut definitions = HashMap::new();

    for entry in dir.find("*.luau").unwrap() {
        let entry_file = entry.as_file().unwrap();
        let entry_name = entry_file.path().file_name().unwrap().to_string_lossy();

        let typedef_name = entry_name.trim_end_matches(".luau");
        let typedef_contents = entry_file.contents().to_vec();

        definitions.insert(typedef_name.to_string(), typedef_contents);
    }

    definitions
}

fn read_typedefs_dir(dir: &Dir<'_>) -> Result<HashMap<String, DefinitionsTree>> {
    let mut definitions = HashMap::new();

    for entry in dir.find("*.luau").unwrap() {
        let entry_file = entry.as_file().unwrap();
        let entry_name = entry_file.path().file_name().unwrap().to_string_lossy();

        let typedef_name = entry_name.trim_end_matches(".luau");
        let typedef_contents = entry_file.contents_utf8().unwrap().to_string();

        let typedef_tree = DefinitionsTree::from_type_definitions(&typedef_contents)?;
        definitions.insert(typedef_name.to_string(), typedef_tree);
    }

    Ok(definitions)
}
