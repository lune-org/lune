use std::ops::{Deref, DerefMut};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    builder::DefinitionsItemBuilder, item::DefinitionsItem, kind::DefinitionsItemKind,
    parser::DefinitionsParser,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefinitionsTree(DefinitionsItem);

#[allow(dead_code)]
impl DefinitionsTree {
    pub fn from_type_definitions<S: AsRef<str>>(type_definitions_contents: S) -> Result<Self> {
        let mut parser = DefinitionsParser::new();
        parser
            .parse(type_definitions_contents)
            .context("Failed to parse type definitions AST")?;
        let top_level_definition_items = parser
            .drain()
            .context("Failed to convert parser items into definition items")?;
        let root = DefinitionsItemBuilder::new()
            .with_kind(DefinitionsItemKind::Root)
            .with_name("<<<ROOT>>>")
            .with_children(&top_level_definition_items)
            .build()?;
        Ok(Self(root))
    }

    #[allow(clippy::unused_self)]
    pub fn is_root(&self) -> bool {
        true
    }

    pub fn into_inner(self) -> DefinitionsItem {
        self.0
    }
}

impl Deref for DefinitionsTree {
    type Target = DefinitionsItem;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DefinitionsTree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
