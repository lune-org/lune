use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DocItemKind {
    Root,
    Table,
    Property,
    Function,
    Description,
    Tag,
}

#[allow(dead_code)]
impl DocItemKind {
    pub fn is_root(self) -> bool {
        self == DocItemKind::Root
    }

    pub fn is_table(self) -> bool {
        self == DocItemKind::Table
    }

    pub fn is_property(self) -> bool {
        self == DocItemKind::Property
    }

    pub fn is_function(self) -> bool {
        self == DocItemKind::Function
    }

    pub fn is_description(self) -> bool {
        self == DocItemKind::Description
    }

    pub fn is_tag(self) -> bool {
        self == DocItemKind::Tag
    }
}

impl fmt::Display for DocItemKind {
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
