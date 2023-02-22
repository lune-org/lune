use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DefinitionsItemKind {
    Root,
    Table,
    Property,
    Function,
    Description,
    Tag,
}

#[allow(dead_code)]
impl DefinitionsItemKind {
    pub fn is_root(self) -> bool {
        self == DefinitionsItemKind::Root
    }

    pub fn is_table(self) -> bool {
        self == DefinitionsItemKind::Table
    }

    pub fn is_property(self) -> bool {
        self == DefinitionsItemKind::Property
    }

    pub fn is_function(self) -> bool {
        self == DefinitionsItemKind::Function
    }

    pub fn is_description(self) -> bool {
        self == DefinitionsItemKind::Description
    }

    pub fn is_tag(self) -> bool {
        self == DefinitionsItemKind::Tag
    }
}

impl fmt::Display for DefinitionsItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Root => "Root",
                Self::Table => "Table",
                Self::Property => "Property",
                Self::Function => "Function",
                Self::Description => "Description",
                Self::Tag => "Tag",
            }
        )
    }
}
