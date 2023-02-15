use serde_json::Value;

use crate::gen::parse_definitions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Json,
    Yaml,
    Luau,
}

impl FileType {
    pub fn from_contents(contents: &str) -> Option<Self> {
        if serde_json::from_str::<Value>(contents).is_ok() {
            Some(Self::Json)
        } else if serde_yaml::from_str::<Value>(contents).is_ok() {
            Some(Self::Yaml)
        } else if parse_definitions(contents).is_ok() {
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
