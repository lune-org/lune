use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::Enum as RbxEnum;
use rbx_reflection::DataType as RbxDataType;

use super::Enum;

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
    /**
        Converts an instance property into an [`EnumItem`] datatype, if the property is known.

        Enums are not strongly typed which means we can not convert directly from a [`rbx_dom_weak::types::Enum`]
        into an `EnumItem` without losing information about its parent [`Enum`] and the `EnumItem` name.

        This constructor exists as a shortcut to perform a [`rbx_reflection_database`] lookup for a particular
        instance class and property to construct a strongly typed `EnumItem` with no loss of information.
    */
    #[allow(dead_code)]
    fn from_instance_property(
        class_name: impl AsRef<str>,
        prop_name: impl AsRef<str>,
        value: u32,
    ) -> Option<Self> {
        let db = rbx_reflection_database::get();
        let prop = db
            .classes
            .get(class_name.as_ref())?
            .properties
            .get(prop_name.as_ref())?;
        let prop_enum = match &prop.data_type {
            RbxDataType::Enum(name) => db.enums.get(name.as_ref()),
            _ => None,
        }?;
        let enum_name = prop_enum.items.iter().find_map(|(name, v)| {
            if v == &value {
                Some(name.to_string())
            } else {
                None
            }
        })?;
        Some(Self {
            parent: prop_enum.into(),
            name: enum_name,
            value,
        })
    }
}

impl LuaUserData for EnumItem {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.name.clone()));
        fields.add_field_method_get("Value", |_, this| Ok(this.value));
        fields.add_field_method_get("EnumType", |_, this| Ok(this.parent.clone()));
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

impl From<EnumItem> for RbxEnum {
    fn from(v: EnumItem) -> Self {
        RbxEnum::from_u32(v.value)
    }
}
