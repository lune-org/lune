use anyhow::{bail, Result};

use super::{
    item::{DefinitionsItem, DefinitionsItemFunctionArg, DefinitionsItemFunctionRet},
    kind::DefinitionsItemKind,
};

#[derive(Debug, Default, Clone)]
pub struct DefinitionsItemBuilder {
    exported: bool,
    kind: Option<DefinitionsItemKind>,
    name: Option<String>,
    meta: Option<String>,
    value: Option<String>,
    children: Vec<DefinitionsItem>,
    args: Vec<DefinitionsItemFunctionArg>,
    rets: Vec<DefinitionsItemFunctionRet>,
}

#[allow(dead_code)]
impl DefinitionsItemBuilder {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn as_exported(mut self) -> Self {
        self.exported = true;
        self
    }

    pub fn with_kind(mut self, kind: DefinitionsItemKind) -> Self {
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

    pub fn with_child(mut self, child: DefinitionsItem) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_children(mut self, children: &[DefinitionsItem]) -> Self {
        self.children.extend_from_slice(children);
        self
    }

    pub fn with_arg(mut self, arg: DefinitionsItemFunctionArg) -> Self {
        self.args.push(arg);
        self
    }

    pub fn with_args(mut self, args: &[DefinitionsItemFunctionArg]) -> Self {
        for arg in args {
            self.args.push(arg.clone());
        }
        self
    }

    pub fn with_ret(mut self, ret: DefinitionsItemFunctionRet) -> Self {
        self.rets.push(ret);
        self
    }

    pub fn with_rets(mut self, rets: &[DefinitionsItemFunctionRet]) -> Self {
        for ret in rets {
            self.rets.push(ret.clone());
        }
        self
    }

    pub fn build(self) -> Result<DefinitionsItem> {
        if let Some(kind) = self.kind {
            let mut children = self.children;
            children.sort();
            Ok(DefinitionsItem {
                exported: self.exported,
                kind,
                name: self.name,
                meta: self.meta,
                value: self.value,
                children,
                args: self.args,
                rets: self.rets,
            })
        } else {
            bail!("Missing doc item kind")
        }
    }
}
