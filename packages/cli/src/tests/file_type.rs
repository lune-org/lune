use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

use crate::gen::DocumentationVisitor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Json,
    Yaml,
    Luau,
}

impl FileType {
    pub fn sniff(contents: &str) -> Option<Self> {
        if serde_json::from_str::<JsonValue>(contents).is_ok() {
            Some(Self::Json)
        } else if serde_yaml::from_str::<YamlValue>(contents).is_ok() {
            Some(Self::Yaml)
        } else if DocumentationVisitor::from_definitions(contents).is_ok() {
            Some(Self::Luau)
        } else {
            None
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            FileType::Json => "json",
            FileType::Yaml => "yaml",
            FileType::Luau => "luau",
        }
    }
}
