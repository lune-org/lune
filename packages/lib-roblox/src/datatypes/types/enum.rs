use core::fmt;

use mlua::prelude::*;
use rbx_reflection::EnumDescriptor;

use super::{super::*, EnumItem};

/**
    An implementation of the [Enum](https://create.roblox.com/docs/reference/engine/datatypes/Enum) Roblox datatype.

    This implements all documented properties, methods & constructors of the Enum class as of March 2023.
*/
#[derive(Debug, Clone)]
pub struct Enum {
    pub(crate) desc: &'static EnumDescriptor<'static>,
}

impl LuaUserData for Enum {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Methods
        methods.add_method("GetEnumItems", |_, this, ()| {
            Ok(this
                .desc
                .items
                .iter()
                .map(|(name, value)| EnumItem {
                    parent: this.clone(),
                    name: name.to_string(),
                    value: *value,
                })
                .collect::<Vec<_>>())
        });
        methods.add_meta_method(LuaMetaMethod::Index, |_, this, name: String| {
            match this.desc.items.get(name.as_str()) {
                Some(value) => Ok(EnumItem {
                    parent: this.clone(),
                    name: name.to_string(),
                    value: *value,
                }),
                None => Err(LuaError::RuntimeError(format!(
                    "The enum item '{}' does not exist for enum '{}'",
                    name, this.desc.name
                ))),
            }
        });
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Enum.{}", self.desc.name)
    }
}

impl PartialEq for Enum {
    fn eq(&self, other: &Self) -> bool {
        self.desc.name == other.desc.name
    }
}

impl From<&'static EnumDescriptor<'static>> for Enum {
    fn from(value: &'static EnumDescriptor<'static>) -> Self {
        Self { desc: value }
    }
}
