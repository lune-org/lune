use std::collections::HashMap;

use anyhow::{Context, Result};
use serde_json::{Map as JsonMap, Value as JsonValue};

use super::doc::{DocsFunctionParamLink, DocumentationVisitor};

pub fn generate_from_type_definitions(contents: &str, namespace: &str) -> Result<String> {
    let visitor = DocumentationVisitor::from_definitions(contents)?;
    /*
        Extract globals, functions, params, returns from the visitor
        Here we will also convert the plain names into proper namespaced names according to the spec at
        https://raw.githubusercontent.com/MaximumADHD/Roblox-Client-Tracker/roblox/api-docs/en-us.json
    */
    let mut map = JsonMap::new();
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
    serde_json::to_string_pretty(&JsonValue::Object(map)).context("Failed to encode docs as json")
}
