use core::fmt;

use mlua::prelude::*;
use rbx_reflection::EnumDescriptor;

use super::{super::*, EnumItem, enum_item_to_lua, opt_enum_item_to_lua};

/**
    An implementation of the [Enum](https://create.roblox.com/docs/reference/engine/datatypes/Enum) Roblox datatype.

    This implements all documented properties, methods & constructors of the Enum class as of May 2026.
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
        methods.add_method("GetEnumItems", |lua, this, ()| {
            let items = this.desc.items.iter().map(|(name, value)| EnumItem {
                parent: this.clone(),
                name: name.to_string(),
                value: *value,
            });
            items
                .map(|item| enum_item_to_lua(lua, item))
                .collect::<LuaResult<Vec<_>>>()
        });
        methods.add_method("FromName", |lua, this, name: LuaBorrowedStr| {
            opt_enum_item_to_lua(lua, EnumItem::from_enum_and_name(this, name))
        });
        methods.add_method("FromValue", |lua, this, value: u32| {
            opt_enum_item_to_lua(lua, EnumItem::from_enum_and_value(this, value))
        });
        methods.add_meta_method(LuaMetaMethod::Index, |lua, this, name: String| {
            match EnumItem::from_enum_and_name(this, &name) {
                Some(item) => enum_item_to_lua(lua, item),
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
