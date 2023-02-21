use std::ops::{Deref, DerefMut};

use anyhow::{Context, Result};

use super::{builder::DocItemBuilder, item::DocItem, kind::DocItemKind, visitor::DocVisitor};

pub struct DocTree(DocItem);

impl DocTree {
    pub fn from_type_definitions<S: AsRef<str>>(type_definitions_contents: S) -> Result<Self> {
        let top_level_items = DocVisitor::visit_type_definitions_str(type_definitions_contents)
            .context("Failed to visit type definitions AST")?;
        let root = DocItemBuilder::new()
            .with_kind(DocItemKind::Root)
            .with_name("<<<ROOT>>>")
            .with_children(&top_level_items)
            .build()?;
        Ok(Self(root))
    }

    #[allow(dead_code, clippy::unused_self)]
    pub fn is_root(&self) -> bool {
        true
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

impl IntoIterator for DocTree {
    type Item = DocItem;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
