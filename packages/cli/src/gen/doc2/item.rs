use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::kind::DocItemKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocItem {
    pub(super) kind: DocItemKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) meta: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) value: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) children: Vec<DocItem>,
}

impl PartialOrd for DocItem {
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

impl Ord for DocItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[allow(dead_code)]
impl DocItem {
    pub fn is_root(&self) -> bool {
        self.kind.is_root()
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

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn get_meta(&self) -> Option<&str> {
        self.meta.as_deref()
    }

    pub fn get_value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

impl IntoIterator for DocItem {
    type Item = DocItem;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}
