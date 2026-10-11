use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::EnumItem as DomEnumItem;

use super::{super::*, Enum};

/**
    An implementation of the [EnumItem](https://create.roblox.com/docs/reference/engine/datatypes/EnumItem) Roblox datatype.

    This implements all documented properties, methods & constructors of the `EnumItem` class as of May 2026.
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
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_get("Value", |_, this| Ok(this.value));
        fields.add_field_method_get("EnumType", |_, this| Ok(this.parent.clone()));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl FromLua for EnumItem {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        if let LuaValue::UserData(ud) = value {
            Ok(ud.borrow::<EnumItem>()?.to_owned())
        } else {
            Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "EnumItem".to_string(),
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

impl From<EnumItem> for DomEnumItem {
    fn from(v: EnumItem) -> Self {
        DomEnumItem {
            ty: v.parent.desc.name.to_string(),
            value: v.value,
        }
    }
}

impl From<DomEnumItem> for EnumItem {
    fn from(value: DomEnumItem) -> Self {
        EnumItem::from_enum_name_and_value(value.ty, value.value)
            .expect("cannot convert rbx_type::EnumItem with unknown type into EnumItem")
    }
}

/*
    Userdata interning

    Enum items are interned in the same way as instances, so that two reads
    of the same enum item give the *same* userdata - matching Roblox, and
    making enum items work correctly as table keys and with rawequal.
*/
const ENUM_ITEM_CACHE_KEY: &str = "__lune_roblox_enum_item_cache";

/**
    Converts an enum item into its canonical lua userdata, creating and caching
    it the first time and returning the same userdata on subsequent calls.

    # Errors

    Errors if creating the userdata or accessing the interning cache fails.
*/
pub(crate) fn enum_item_to_lua(lua: &Lua, item: EnumItem) -> LuaResult<LuaValue> {
    let cache = interning_cache(lua, ENUM_ITEM_CACHE_KEY)?;
    let key = item.to_string();
    if let Ok(LuaValue::UserData(existing)) = cache.get::<LuaValue>(key.as_str()) {
        return Ok(LuaValue::UserData(existing));
    }
    let userdata = lua.create_userdata(item)?;
    cache.set(key, &userdata)?;
    Ok(LuaValue::UserData(userdata))
}

/**
    Converts an optional enum item into lua, returning [`LuaValue::Nil`] for [`None`].

    # Errors

    Errors if creating the userdata or accessing the interning cache fails.
*/
pub(crate) fn opt_enum_item_to_lua(lua: &Lua, item: Option<EnumItem>) -> LuaResult<LuaValue> {
    match item {
        Some(item) => enum_item_to_lua(lua, item),
        None => Ok(LuaValue::Nil),
    }
}
