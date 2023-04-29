use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::kind::DefinitionsItemKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionsItemFunctionArg {
    pub name: String,
    pub typedef: String,
    pub typedef_simple: String,
}

impl DefinitionsItemFunctionArg {
    pub fn new<N, T, TS>(name: N, typedef: T, typedef_simple: TS) -> Self
    where
        N: Into<String>,
        T: Into<String>,
        TS: Into<String>,
    {
        Self {
            name: name.into(),
            typedef: typedef.into(),
            typedef_simple: typedef_simple.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionsItemFunctionRet {
    pub typedef: String,
    pub typedef_simple: String,
}

impl DefinitionsItemFunctionRet {
    pub fn new<T, TS>(typedef: T, typedef_simple: TS) -> Self
    where
        T: Into<String>,
        TS: Into<String>,
    {
        Self {
            typedef: typedef.into(),
            typedef_simple: typedef_simple.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionsItem {
    #[serde(skip_serializing_if = "skip_serialize_is_false")]
    pub(super) exported: bool,
    pub(super) kind: DefinitionsItemKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) meta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) value: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) children: Vec<DefinitionsItem>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) args: Vec<DefinitionsItemFunctionArg>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) rets: Vec<DefinitionsItemFunctionRet>,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn skip_serialize_is_false(b: &bool) -> bool {
    !b
}

impl PartialOrd for DefinitionsItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.kind.partial_cmp(&other.kind).unwrap() {
            Ordering::Equal => {}
            ord => return Some(ord),
        }
        match self.name.partial_cmp(&other.name).unwrap() {
            Ordering::Equal => {}
            ord => return Some(ord),
        }
        match (&self.value, &other.value) {
            (Some(value_self), Some(value_other)) => {
                match value_self.partial_cmp(value_other).unwrap() {
                    Ordering::Equal => {}
                    ord => return Some(ord),
                }
            }
            (Some(_), None) => return Some(Ordering::Less),
            (None, Some(_)) => return Some(Ordering::Greater),
            (None, None) => {}
        }
        Some(Ordering::Equal)
    }
}

impl Ord for DefinitionsItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[allow(dead_code)]
impl DefinitionsItem {
    pub fn is_exported(&self) -> bool {
        self.exported
    }

    pub fn is_root(&self) -> bool {
        self.kind.is_root()
    }

    pub fn is_table(&self) -> bool {
        self.kind.is_table()
    }

    pub fn is_property(&self) -> bool {
        self.kind.is_property()
    }

    pub fn is_function(&self) -> bool {
        self.kind.is_function()
    }

    pub fn is_description(&self) -> bool {
        self.kind.is_description()
    }

    pub fn is_tag(&self) -> bool {
        self.kind.is_tag()
    }

    pub fn kind(&self) -> DefinitionsItemKind {
        self.kind
    }

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn get_meta(&self) -> Option<&str> {
        self.meta.as_deref()
    }

    pub fn get_value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    pub fn children(&self) -> &[DefinitionsItem] {
        &self.children
    }

    pub fn args(&self) -> Vec<&DefinitionsItemFunctionArg> {
        self.args.iter().collect()
    }

    pub fn rets(&self) -> Vec<&DefinitionsItemFunctionRet> {
        self.rets.iter().collect()
    }
}
