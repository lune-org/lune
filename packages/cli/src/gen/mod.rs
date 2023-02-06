use std::collections::HashMap;

use anyhow::Result;
use regex::Regex;
use serde_json::{Map, Value};

use full_moon::{parse as parse_luau_ast, visitors::Visitor};

mod doc;
mod tag;
mod visitor;

use self::{doc::DocsFunctionParamLink, visitor::DocumentationVisitor};

fn parse_definitions(contents: &str) -> Result<DocumentationVisitor> {
    let (regex, replacement) = (
        Regex::new(r#"declare (?P<n>\w+): \{"#).unwrap(),
        r#"export type $n = {"#,
    );
    let defs_ast = parse_luau_ast(&regex.replace_all(contents, replacement))?;
    let mut visitor = DocumentationVisitor::new();
    visitor.visit_ast(&defs_ast);
    Ok(visitor)
}

pub fn generate_docs_json_from_definitions(contents: &str, namespace: &str) -> Result<Value> {
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
    Ok(Value::Object(map))
}
