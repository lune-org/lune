use core::fmt;

use mlua::prelude::*;
use rbx_reflection::EnumDescriptor;

use super::{super::*, EnumItem};

/**
    An implementation of the [Enum](https://create.roblox.com/docs/reference/engine/datatypes/Enum) Roblox datatype.

    This implements all documented properties, methods & constructors of the Enum class as of October 2025.
*/
#[derive(Debug, Clone)]
pub struct Enum {
    pub(crate) desc: &'static EnumDescriptor<'static>,
}

impl Enum {
    pub(crate) fn from_name(name: impl AsRef<str>) -> Option<Self> {
        let db = rbx_reflection_database::get().unwrap();
        db.enums.get(name.as_ref()).map(Enum::from)
    }
}

impl LuaUserData for Enum {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
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
            match EnumItem::from_enum_and_name(this, &name) {
                Some(item) => Ok(item),
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
