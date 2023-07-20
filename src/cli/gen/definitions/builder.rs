use anyhow::{bail, Result};

use super::{
    item::{DefinitionsItem, DefinitionsItemFunctionArg, DefinitionsItemFunctionRet},
    kind::DefinitionsItemKind,
};

#[derive(Debug, Default, Clone)]
pub struct DefinitionsItemBuilder {
    exported: bool,
    kind: Option<DefinitionsItemKind>,
    typ: Option<String>,
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

    pub fn with_type(mut self, typ: String) -> Self {
        self.typ = Some(typ);
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
            children.sort_by(|left, right| left.name.cmp(&right.name));
            Ok(DefinitionsItem {
                exported: self.exported,
                kind,
                typ: self.typ,
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

impl From<&DefinitionsItem> for DefinitionsItemBuilder {
    fn from(value: &DefinitionsItem) -> Self {
        Self {
            exported: value.exported,
            kind: Some(value.kind),
            typ: value.typ.clone(),
            name: value.name.clone(),
            meta: value.meta.clone(),
            value: value.value.clone(),
            children: value.children.clone(),
            args: value.args.clone(),
            rets: value.rets.clone(),
        }
    }
}
