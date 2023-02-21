use anyhow::{bail, Result};

use super::{item::DocItem, kind::DocItemKind};

#[derive(Debug, Default, Clone)]
pub struct DocItemBuilder {
    kind: Option<DocItemKind>,
    name: Option<String>,
    value: Option<String>,
    children: Vec<DocItem>,
}

#[allow(dead_code)]
impl DocItemBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn with_kind(mut self, kind: DocItemKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn with_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.name = Some(name.as_ref().to_string());
        self
    }

    pub fn with_value<S: AsRef<str>>(mut self, value: S) -> Self {
        self.value = Some(value.as_ref().to_string());
        self
    }

    pub fn with_child(mut self, child: DocItem) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_children(mut self, children: &[DocItem]) -> Self {
        self.children.extend_from_slice(children);
        self
    }

    pub fn build(self) -> Result<DocItem> {
        if let Some(kind) = self.kind {
            if let Some(name) = self.name {
                let mut children = self.children;
                children.sort();
                Ok(DocItem {
                    kind,
                    name,
                    value: self.value,
                    children,
                })
            } else {
                bail!("Missing doc item name")
            }
        } else {
            bail!("Missing doc item kind")
        }
    }
}
