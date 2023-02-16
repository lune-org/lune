use anyhow::{Context, Result};
use serde_yaml::Value as YamlValue;

use super::doc::DocumentationVisitor;

pub fn generate_from_type_definitions(contents: &str) -> Result<String> {
    let _visitor = DocumentationVisitor::from_definitions(contents)?;
    serde_yaml::to_string(&YamlValue::Null).context("Failed to encode docs as json")
}
