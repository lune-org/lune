use anyhow::{bail, Result};

use super::{item::DocItem, kind::DocItemKind};

#[derive(Debug, Default, Clone)]
pub struct DocItemBuilder {
    kind: Option<DocItemKind>,
    name: Option<String>,
    meta: Option<String>,
    value: Option<String>,
    children: Vec<DocItem>,
    arg_types: Vec<String>,
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

    pub fn with_meta<S: AsRef<str>>(mut self, meta: S) -> Self {
        self.meta = Some(meta.as_ref().to_string());
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

    pub fn with_arg_type<S: AsRef<str>>(mut self, arg_type: S) -> Self {
        self.arg_types.push(arg_type.as_ref().to_string());
        self
    }

    pub fn with_arg_types<S: AsRef<str>>(mut self, arg_types: &[S]) -> Self {
        for arg_type in arg_types {
            self.arg_types.push(arg_type.as_ref().to_string());
        }
        self
    }

    pub fn build(self) -> Result<DocItem> {
        if let Some(kind) = self.kind {
            let mut children = self.children;
            children.sort();
            Ok(DocItem {
                kind,
                name: self.name,
                meta: self.meta,
                value: self.value,
                children,
                arg_types: self.arg_types,
            })
        } else {
            bail!("Missing doc item kind")
        }
    }
}
