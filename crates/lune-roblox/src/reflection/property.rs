use std::fmt;

use mlua::prelude::*;

use rbx_reflection::{ClassDescriptor, PropertyDescriptor};

use super::utils::*;
use crate::datatypes::{userdata_impl_eq, userdata_impl_to_string};

type DbClass = &'static ClassDescriptor<'static>;
type DbProp = &'static PropertyDescriptor<'static>;

/**
    A wrapper for [`rbx_reflection::PropertyDescriptor`] that
    also provides access to the property descriptor from lua.
*/
#[derive(Debug, Clone, Copy)]
pub struct DatabaseProperty(DbClass, DbProp);

impl DatabaseProperty {
    pub(crate) fn new(inner: DbClass, inner_prop: DbProp) -> Self {
        Self(inner, inner_prop)
    }

    /**
        Get the name of this property.
    */
    pub fn get_name(&self) -> String {
        self.1.name.to_string()
    }

    /**
        Get the datatype name of the property.

        For normal datatypes this will be a string such as `string`, `Color3`, ...

        For enums this will be a string formatted as `Enum.EnumName`.
    */
    pub fn get_datatype_name(&self) -> String {
        data_type_to_str(self.1.data_type.clone())
    }

    /**
        Get the scriptability of this property, meaning if it can be written / read at runtime.

        All properties are writable and readable in Lune even if scriptability is not.
    */
    pub fn get_scriptability_str(&self) -> &'static str {
        scriptability_to_str(&self.1.scriptability)
    }

    /**
        Get all known tags describing the property.

        These include information such as if the property can be replicated to players
        at runtime, if the property should be hidden in Roblox Studio, and more.
    */
    pub fn get_tags_str(&self) -> Vec<&'static str> {
        self.1
            .tags
            .iter()
            .map(property_tag_to_str)
            .collect::<Vec<_>>()
    }
}

impl LuaUserData for DatabaseProperty {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.get_name()));
        fields.add_field_method_get("Datatype", |_, this| Ok(this.get_datatype_name()));
        fields.add_field_method_get("Scriptability", |_, this| Ok(this.get_scriptability_str()));
        fields.add_field_method_get("Tags", |_, this| Ok(this.get_tags_str()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl PartialEq for DatabaseProperty {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name && self.1.name == other.1.name
    }
}

impl fmt::Display for DatabaseProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ReflectionDatabaseProperty({} > {})",
            self.0.name, self.1.name
        )
    }
}
