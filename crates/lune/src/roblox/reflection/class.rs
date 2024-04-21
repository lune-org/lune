use core::fmt;
use std::collections::HashMap;

use mlua::prelude::*;

use rbx_dom_weak::types::Variant as DomVariant;
use rbx_reflection::{ClassDescriptor, DataType};

use super::{property::DatabaseProperty, utils::*};
use crate::roblox::datatypes::{
    conversion::DomValueToLua, types::EnumItem, userdata_impl_eq, userdata_impl_to_string,
};

type DbClass = &'static ClassDescriptor<'static>;

/**
    A wrapper for [`rbx_reflection::ClassDescriptor`] that
    also provides access to the class descriptor from lua.
*/
#[derive(Debug, Clone, Copy)]
pub struct DatabaseClass(DbClass);

impl DatabaseClass {
    pub(crate) fn new(inner: DbClass) -> Self {
        Self(inner)
    }

    /**
        Get the name of this class.
    */
    pub fn get_name(&self) -> String {
        self.0.name.to_string()
    }

    /**
        Get the superclass (parent class) of this class.

        May be `None` if no parent class exists.
    */
    pub fn get_superclass(&self) -> Option<String> {
        let sup = self.0.superclass.as_ref()?;
        Some(sup.to_string())
    }

    /**
        Get all known properties for this class.
    */
    pub fn get_properties(&self) -> HashMap<String, DatabaseProperty> {
        self.0
            .properties
            .iter()
            .map(|(name, prop)| (name.to_string(), DatabaseProperty::new(self.0, prop)))
            .collect()
    }

    /**
        Get all default values for properties of this class.
    */
    pub fn get_defaults(&self) -> HashMap<String, DomVariant> {
        self.0
            .default_properties
            .iter()
            .map(|(name, prop)| (name.to_string(), prop.clone()))
            .collect()
    }

    /**
        Get all tags describing the class.

        These include information such as if the class can be replicated
        to players at runtime, and top-level class categories.
    */
    pub fn get_tags_str(&self) -> Vec<&'static str> {
        self.0.tags.iter().map(class_tag_to_str).collect::<Vec<_>>()
    }
}

impl LuaUserData for DatabaseClass {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.get_name()));
        fields.add_field_method_get("Superclass", |_, this| Ok(this.get_superclass()));
        fields.add_field_method_get("Properties", |_, this| Ok(this.get_properties()));
        fields.add_field_method_get("DefaultProperties", |lua, this| {
            let defaults = this.get_defaults();
            let mut map = HashMap::with_capacity(defaults.len());
            for (name, prop) in defaults {
                let value = if let DomVariant::Enum(enum_value) = prop {
                    make_enum_value(this.0, &name, enum_value.to_u32())
                        .and_then(|e| e.into_lua(lua))
                } else {
                    LuaValue::dom_value_to_lua(lua, &prop).into_lua_err()
                };
                if let Ok(value) = value {
                    map.insert(name, value);
                }
            }
            Ok(map)
        });
        fields.add_field_method_get("Tags", |_, this| Ok(this.get_tags_str()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl PartialEq for DatabaseClass {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
    }
}

impl fmt::Display for DatabaseClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReflectionDatabaseClass({})", self.0.name)
    }
}

fn find_enum_name(inner: DbClass, name: impl AsRef<str>) -> Option<String> {
    inner.properties.iter().find_map(|(prop_name, prop_info)| {
        if prop_name == name.as_ref() {
            if let DataType::Enum(enum_name) = &prop_info.data_type {
                Some(enum_name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn make_enum_value(inner: DbClass, name: impl AsRef<str>, value: u32) -> LuaResult<EnumItem> {
    let name = name.as_ref();
    let enum_name = find_enum_name(inner, name).ok_or_else(|| {
        LuaError::RuntimeError(format!(
            "Failed to get default property '{}' - No enum descriptor was found",
            name
        ))
    })?;
    EnumItem::from_enum_name_and_value(&enum_name, value).ok_or_else(|| {
        LuaError::RuntimeError(format!(
            "Failed to get default property '{}' - Enum.{} does not contain numeric value {}",
            name, enum_name, value
        ))
    })
}
