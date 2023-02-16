use std::{collections::HashMap, fmt::Write, path::PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{Map, Value};

use full_moon::{parse as parse_luau_ast, visitors::Visitor};
use tokio::fs::{create_dir_all, write};

mod doc;
mod tag;
mod visitor;

const GENERATED_COMMENT_TAG: &str = "@generated with lune-cli";

use self::{doc::DocsFunctionParamLink, visitor::DocumentationVisitor};

pub fn parse_definitions(contents: &str) -> Result<DocumentationVisitor> {
    // TODO: Properly handle the "declare class" syntax, for now we just skip it
    let mut no_declares = contents.to_string();
    while let Some(dec) = no_declares.find("\ndeclare class") {
        let end = no_declares.find("\nend").unwrap();
        let before = &no_declares[0..dec];
        let after = &no_declares[end + 4..];
        no_declares = format!("{before}{after}");
    }
    let (regex, replacement) = (
        Regex::new(r#"declare (?P<n>\w+): "#).unwrap(),
        r#"export type $n = "#,
    );
    let defs_ast = parse_luau_ast(&regex.replace_all(&no_declares, replacement))?;
    let mut visitor = DocumentationVisitor::new();
    visitor.visit_ast(&defs_ast);
    Ok(visitor)
}

pub fn generate_docs_json_from_definitions(contents: &str, namespace: &str) -> Result<String> {
    let visitor = parse_definitions(contents)?;
    /*
        Extract globals, functions, params, returns from the visitor
        Here we will also convert the plain names into proper namespaced names according to the spec at
        https://raw.githubusercontent.com/MaximumADHD/Roblox-Client-Tracker/roblox/api-docs/en-us.json
    */
    let mut map = Map::new();
    for (name, mut doc) in visitor.globals {
        doc.keys = doc
            .keys
            .iter()
            .map(|(key, value)| (key.clone(), format!("@{namespace}/{name}.{value}")))
            .collect::<HashMap<String, String>>();
        map.insert(format!("@{namespace}/{name}"), serde_json::to_value(doc)?);
    }
    for (name, mut doc) in visitor.functions {
        doc.params = doc
            .params
            .iter()
            .map(|param| DocsFunctionParamLink {
                name: param.name.clone(),
                documentation: format!(
                    "@{namespace}/{}.{name}/param/{}",
                    doc.global_name, param.documentation
                ),
            })
            .collect::<Vec<_>>();
        doc.returns = doc
            .returns
            .iter()
            .map(|ret| format!("@{namespace}/{}.{name}/return/{ret}", doc.global_name))
            .collect::<Vec<_>>();
        map.insert(
            format!("@{namespace}/{}.{name}", doc.global_name),
            serde_json::to_value(doc)?,
        );
    }
    for (name, doc) in visitor.params {
        map.insert(
            format!(
                "@{namespace}/{}.{}/param/{name}",
                doc.global_name, doc.function_name
            ),
            serde_json::to_value(doc)?,
        );
    }
    for (name, doc) in visitor.returns {
        map.insert(
            format!(
                "@{namespace}/{}.{}/return/{name}",
                doc.global_name, doc.function_name
            ),
            serde_json::to_value(doc)?,
        );
    }
    serde_json::to_string_pretty(&Value::Object(map)).context("Failed to encode docs as json")
}

pub async fn generate_wiki_dir_from_definitions(contents: &str) -> Result<()> {
    // Create the wiki dir at the repo root
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .canonicalize()
        .unwrap();
    create_dir_all(&root.join("wiki"))
        .await
        .context("Failed to create wiki dir")?;
    let visitor = parse_definitions(contents)?;
    for global in &visitor.globals {
        // Create the dir for this global
        let global_dir_path = root.join("wiki").join("api-reference").join(&global.0);
        create_dir_all(&global_dir_path)
            .await
            .context("Failed to create doc dir for global")?;
        // Create the markdown docs file for this global
        let mut contents = String::new();
        writeln!(contents, "<!-- {GENERATED_COMMENT_TAG} -->\n")?;
        writeln!(contents, "# **{}**\n", global.0)?;
        writeln!(contents, "{}\n", global.1.documentation)?;
        if !global.1.code_sample.is_empty() {
            writeln!(contents, "{}", global.1.code_sample)?;
        }
        let funcs = visitor
            .functions
            .iter()
            .filter(|f| f.1.global_name == global.0)
            .collect::<Vec<_>>();
        if !funcs.is_empty() {
            writeln!(contents, "## Functions\n")?;
            for func in funcs {
                writeln!(contents, "### {}\n", func.0)?;
                writeln!(contents, "{}\n", func.1.documentation)?;
                if !func.1.code_sample.is_empty() {
                    writeln!(contents, "{}", func.1.code_sample)?;
                }
            }
        }
        // Write the file in the dir, with the same
        // name as the dir to create an "index" page
        write(&global_dir_path.join(format!("{}.md", &global.0)), contents)
            .await
            .context("Failed to create doc file for global")?;
    }
    Ok(())
}
