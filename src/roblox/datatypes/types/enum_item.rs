use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::Enum as DomEnum;

use super::{super::*, Enum};

/**
    An implementation of the [EnumItem](https://create.roblox.com/docs/reference/engine/datatypes/EnumItem) Roblox datatype.

    This implements all documented properties, methods & constructors of the EnumItem class as of March 2023.
*/
#[derive(Debug, Clone)]
pub struct EnumItem {
    pub(crate) parent: Enum,
    pub(crate) name: String,
    pub(crate) value: u32,
}

impl EnumItem {
    pub(crate) fn from_enum_and_name(parent: &Enum, name: impl AsRef<str>) -> Option<Self> {
        let enum_name = name.as_ref();
        parent.desc.items.iter().find_map(|(name, v)| {
            if *name == enum_name {
                Some(Self {
                    parent: parent.clone(),
                    name: enum_name.to_string(),
                    value: *v,
                })
            } else {
                None
            }
        })
    }

    pub(crate) fn from_enum_and_value(parent: &Enum, value: u32) -> Option<Self> {
        parent.desc.items.iter().find_map(|(name, v)| {
            if *v == value {
                Some(Self {
                    parent: parent.clone(),
                    name: name.to_string(),
                    value,
                })
            } else {
                None
            }
        })
    }

    pub(crate) fn from_enum_name_and_name(
        enum_name: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> Option<Self> {
        let parent = Enum::from_name(enum_name)?;
        Self::from_enum_and_name(&parent, name)
    }

    pub(crate) fn from_enum_name_and_value(enum_name: impl AsRef<str>, value: u32) -> Option<Self> {
        let parent = Enum::from_name(enum_name)?;
        Self::from_enum_and_value(&parent, value)
    }
}

impl LuaUserData for EnumItem {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_get("Value", |_, this| Ok(this.value));
        fields.add_field_method_get("EnumType", |_, this| Ok(this.parent.clone()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl<'lua> FromLua<'lua> for EnumItem {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        if let LuaValue::UserData(ud) = value {
            Ok(ud.borrow::<EnumItem>()?.to_owned())
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "EnumItem",
                message: None,
            })
        }
    }
}

impl fmt::Display for EnumItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.parent, self.name)
    }
}

impl PartialEq for EnumItem {
    fn eq(&self, other: &Self) -> bool {
        self.parent == other.parent && self.value == other.value
    }
}

impl From<EnumItem> for DomEnum {
    fn from(v: EnumItem) -> Self {
        DomEnum::from_u32(v.value)
    }
}
