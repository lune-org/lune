use anyhow::{Context, Result};
use serde_json::{Map as JsonMap, Value as JsonValue};

use super::definitions::{DefinitionsItem, DefinitionsItemTag, DefinitionsTree};

static KEY_DOCUMENTATION: &str = "documentation";
static KEY_KEYS: &str = "keys";
static KEY_NAME: &str = "name";
static KEY_CODE_SAMPLE: &str = "code_sample";
static KEY_LEARN_MORE_LINK: &str = "learn_more_link";
static VALUE_EMPTY: &str = "";

pub fn generate_from_type_definitions(contents: &str, namespace: &str) -> Result<String> {
    let tree = DefinitionsTree::from_type_definitions(contents)?;
    /*
        Extract globals, functions, params, returns from the type definitions tree
        Here we will also convert the plain names into proper namespaced names according to the spec at
        https://raw.githubusercontent.com/MaximumADHD/Roblox-Client-Tracker/roblox/api-docs/en-us.json
    */
    let mut map = JsonMap::new();
    // Go over all the exported classes first (globals)
    let exported_items = tree.children().iter().filter(|item| {
        item.is_exported()
            && (item.is_function()
                || item.children().iter().any(|item_child| {
                    item_child.is_tag() && item_child.get_name().unwrap() == "class"
                }))
    });
    for item in exported_items {
        parse_and_insert(&mut map, item, namespace, None)?;
    }
    // Go over the rest, these will be global types
    // that exported items are referencing somewhere
    serde_json::to_string_pretty(&JsonValue::Object(map)).context("Failed to encode docs as json")
}

#[allow(clippy::too_many_lines)]
fn parse_and_insert(
    map: &mut JsonMap<String, JsonValue>,
    item: &DefinitionsItem,
    namespace: &str,
    parent: Option<&DefinitionsItem>,
) -> Result<()> {
    let mut item_map = JsonMap::new();
    let item_name = item
        .get_name()
        .with_context(|| format!("Missing name for doc item: {item:#?}"))?;
    // Include parent name in full name, unless there is no parent (top-level global)
    let item_name_full = match parent {
        Some(parent) => format!(
            "{}.{item_name}",
            parent
                .get_name()
                .with_context(|| format!("Missing parent name for doc item: {item:#?}"))?
        ),
        None => item_name.to_string(),
    };
    // Try to parse params & returns to use later
    let mut params = Vec::new();
    let mut returns = Vec::new();
    if item.is_function() {
        // Map and separate found tags into params & returns
        let mut tags = item
            .children()
            .iter()
            .filter_map(|child| {
                if let Ok(tag) = DefinitionsItemTag::try_from(child) {
                    Some(tag)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        for tag in tags.drain(..) {
            if tag.is_param() {
                params.push(tag);
            } else if tag.is_return() {
                returns.push(tag);
            }
        }
    }
    // Try to parse the description for this typedef item, if it has one,
    // insert description + code sample + learn more link if they exist
    if let Some(description) = item.children().iter().find(|child| child.is_description()) {
        let (description, code_sample, learn_more_link) = try_parse_description_for_docs(
            description
                .get_value()
                .context("Missing description value for doc item")?
                .to_string(),
        );
        item_map.insert(
            KEY_DOCUMENTATION.to_string(),
            JsonValue::String(description),
        );
        if let Some(code_sample) = code_sample {
            item_map.insert(KEY_CODE_SAMPLE.to_string(), JsonValue::String(code_sample));
        } else {
            item_map.insert(
                KEY_CODE_SAMPLE.to_string(),
                JsonValue::String(VALUE_EMPTY.to_string()),
            );
        }
        if let Some(learn_more_link) = learn_more_link {
            item_map.insert(
                KEY_LEARN_MORE_LINK.to_string(),
                JsonValue::String(learn_more_link),
            );
        } else {
            item_map.insert(
                KEY_LEARN_MORE_LINK.to_string(),
                JsonValue::String(VALUE_EMPTY.to_string()),
            );
        }
    }
    /*
        If the typedef item is a table, we should include keys
        which are references from this global to its members,
        then we should parse its members and add them in

        If it is a function, we should parse its params and args,
        make links to them in this object, and then add them in as
        separate items into the globals map, with their documentation
    */
    if item.is_table() {
        let mut keys = item
            .children()
            .iter()
            .filter_map(|child| {
                if child.is_property() || child.is_table() || child.is_function() {
                    Some(child.get_name().expect("Missing name for doc item child"))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if keys.is_empty() {
            item_map.insert(KEY_KEYS.to_string(), JsonValue::Object(JsonMap::new()));
        } else {
            let mut keys_map = JsonMap::new();
            for key in keys.drain(..) {
                keys_map.insert(
                    key.to_string(),
                    JsonValue::String(format!("@{namespace}/{item_name_full}.{key}")),
                );
            }
            item_map.insert(KEY_KEYS.to_string(), JsonValue::Object(keys_map));
        }
    } else if item.is_function() {
        // Add links to params
        if params.is_empty() {
            item_map.insert("params".to_string(), JsonValue::Array(vec![]));
        } else {
            let mut params_vec = Vec::new();
            for (index, param) in params.iter().enumerate() {
                let mut param_map = JsonMap::new();
                if let DefinitionsItemTag::Param((name, _)) = param {
                    param_map.insert(KEY_NAME.to_string(), JsonValue::String(name.to_string()));
                    param_map.insert(
                        KEY_DOCUMENTATION.to_string(),
                        JsonValue::String(format!("@{namespace}/{item_name_full}/param/{index}")),
                    );
                }
                params_vec.push(JsonValue::Object(param_map));
            }
            item_map.insert("params".to_string(), JsonValue::Array(params_vec));
        }
        // Add links to returns
        if returns.is_empty() {
            item_map.insert("returns".to_string(), JsonValue::Array(vec![]));
        } else {
            let mut returns_vec = Vec::new();
            for (index, _) in returns.iter().enumerate() {
                returns_vec.push(JsonValue::String(format!(
                    "@{namespace}/{item_name_full}/return/{index}"
                )));
            }
            item_map.insert("returns".to_string(), JsonValue::Array(returns_vec));
        }
    }
    map.insert(
        format!("@{namespace}/{item_name_full}"),
        JsonValue::Object(item_map),
    );
    if item.is_table() {
        for child in item
            .children()
            .iter()
            .filter(|child| !child.is_description() && !child.is_tag())
        {
            parse_and_insert(map, child, namespace, Some(item))?;
        }
    } else if item.is_function() {
        // FIXME: It seems the order of params and returns here is not
        // deterministic, they can be unordered which leads to confusing docs
        for (index, param) in params.iter().enumerate() {
            let mut param_map = JsonMap::new();
            if let DefinitionsItemTag::Param((_, doc)) = param {
                param_map.insert(
                    KEY_DOCUMENTATION.to_string(),
                    JsonValue::String(format!("{doc}\n\n---\n")),
                );
            }
            map.insert(
                format!("@{namespace}/{item_name_full}/param/{index}"),
                JsonValue::Object(param_map),
            );
        }
        for (index, ret) in returns.iter().enumerate() {
            let mut return_map = JsonMap::new();
            if let DefinitionsItemTag::Return(doc) = ret {
                return_map.insert(
                    KEY_DOCUMENTATION.to_string(),
                    JsonValue::String(doc.to_string()),
                );
            }
            map.insert(
                format!("@{namespace}/{item_name_full}/return/{index}"),
                JsonValue::Object(return_map),
            );
        }
    }
    Ok(())
}

fn try_parse_description_for_docs(description: String) -> (String, Option<String>, Option<String>) {
    // TODO: Implement this
    (description, None, None)
}
