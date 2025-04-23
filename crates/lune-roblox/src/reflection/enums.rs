use std::{collections::HashMap, fmt};

use mlua::prelude::*;

use rbx_reflection::EnumDescriptor;

use crate::datatypes::{userdata_impl_eq, userdata_impl_to_string};

type DbEnum = &'static EnumDescriptor<'static>;

/**
    A wrapper for [`rbx_reflection::EnumDescriptor`] that
    also provides access to the class descriptor from lua.
*/
#[derive(Debug, Clone, Copy)]
pub struct DatabaseEnum(DbEnum);

impl DatabaseEnum {
    pub(crate) fn new(inner: DbEnum) -> Self {
        Self(inner)
    }

    /**
        Get the name of this enum.
    */
    #[must_use]
    pub fn get_name(&self) -> String {
        self.0.name.to_string()
    }

    /**
        Get all known members of this enum.

        Note that this is a direct map of name -> enum values,
        and does not actually use the `EnumItem` datatype itself.
    */
    #[must_use]
    pub fn get_items(&self) -> HashMap<String, u32> {
        self.0
            .items
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect()
    }
}

impl LuaUserData for DatabaseEnum {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Name", |_, this| Ok(this.get_name()));
        fields.add_field_method_get("Items", |_, this| Ok(this.get_items()));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl PartialEq for DatabaseEnum {
    fn eq(&self, other: &Self) -> bool {
        self.0.name == other.0.name
    }
}

impl fmt::Display for DatabaseEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReflectionDatabaseEnum({})", self.0.name)
    }
}
