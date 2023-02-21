use std::ops::{Deref, DerefMut};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    builder::DocItemBuilder, item::DocItem, kind::DocItemKind,
    parser::parse_type_definitions_into_doc_items,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocTree(DocItem);

#[allow(dead_code)]
impl DocTree {
    pub fn from_type_definitions<S: AsRef<str>>(type_definitions_contents: S) -> Result<Self> {
        let top_level_items = parse_type_definitions_into_doc_items(type_definitions_contents)
            .context("Failed to visit type definitions AST")?;
        let root = DocItemBuilder::new()
            .with_kind(DocItemKind::Root)
            .with_name("<<<ROOT>>>")
            .with_children(&top_level_items)
            .build()?;
        Ok(Self(root))
    }

    #[allow(clippy::unused_self)]
    pub fn is_root(&self) -> bool {
        true
    }

    pub fn into_inner(self) -> DocItem {
        self.0
    }
}

impl Deref for DocTree {
    type Target = DocItem;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DocTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
