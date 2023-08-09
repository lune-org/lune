use std::fmt;

use mlua::prelude::*;

use rbx_reflection::ReflectionDatabase;

use crate::roblox::datatypes::userdata_impl_eq;

mod class;
mod enums;
mod property;
mod utils;

pub use class::DatabaseClass;
pub use enums::DatabaseEnum;
pub use property::DatabaseProperty;

use super::datatypes::userdata_impl_to_string;

type Db = &'static ReflectionDatabase<'static>;

/**
    A wrapper for [`rbx_reflection::ReflectionDatabase`] that
    also provides access to the reflection database from lua.
*/
#[derive(Debug, Clone, Copy)]
pub struct Database(Db);

impl Database {
    /**
        Creates a new database struct, referencing the bundled reflection database.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Get the version string of the database.

        This will follow the format `x.y.z.w`, which most
        commonly looks something like `0.567.0.123456789`.
    */
    pub fn get_version(&self) -> String {
        let [x, y, z, w] = self.0.version;
        format!("{x}.{y}.{z}.{w}")
    }

    /**
        Retrieves a list of all currently known enum names.
    */
    pub fn get_enum_names(&self) -> Vec<String> {
        self.0.enums.keys().map(|e| e.to_string()).collect()
    }

    /**
        Retrieves a list of all currently known class names.
    */
    pub fn get_class_names(&self) -> Vec<String> {
        self.0.classes.keys().map(|e| e.to_string()).collect()
    }

    /**
        Gets an enum with the exact given name, if one exists.
    */
    pub fn get_enum(&self, name: impl AsRef<str>) -> Option<DatabaseEnum> {
        let e = self.0.enums.get(name.as_ref())?;
        Some(DatabaseEnum::new(e))
    }

    /**
        Gets a class with the exact given name, if one exists.
    */
    pub fn get_class(&self, name: impl AsRef<str>) -> Option<DatabaseClass> {
        let c = self.0.classes.get(name.as_ref())?;
        Some(DatabaseClass::new(c))
    }

    /**
        Finds an enum with the given name.

        This will use case-insensitive matching and ignore leading and trailing whitespace.
    */
    pub fn find_enum(&self, name: impl AsRef<str>) -> Option<DatabaseEnum> {
        let name = name.as_ref().trim().to_lowercase();
        let (ename, _) = self
            .0
            .enums
            .iter()
            .find(|(ename, _)| ename.trim().to_lowercase() == name)?;
        self.get_enum(ename)
    }

    /**
        Finds a class with the given name.

        This will use case-insensitive matching and ignore leading and trailing whitespace.
    */
    pub fn find_class(&self, name: impl AsRef<str>) -> Option<DatabaseClass> {
        let name = name.as_ref().trim().to_lowercase();
        let (cname, _) = self
            .0
            .classes
            .iter()
            .find(|(cname, _)| cname.trim().to_lowercase() == name)?;
        self.get_class(cname)
    }
}

impl LuaUserData for Database {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Version", |_, this| Ok(this.get_version()))
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_method("GetEnumNames", |_, this, _: ()| Ok(this.get_enum_names()));
        methods.add_method("GetClassNames", |_, this, _: ()| Ok(this.get_class_names()));
        methods.add_method("GetEnum", |_, this, name: String| Ok(this.get_enum(name)));
        methods.add_method("GetClass", |_, this, name: String| Ok(this.get_class(name)));
        methods.add_method("FindEnum", |_, this, name: String| Ok(this.find_enum(name)));
        methods.add_method("FindClass", |_, this, name: String| {
            Ok(this.find_class(name))
        });
    }
}

impl Default for Database {
    fn default() -> Self {
        Self(rbx_reflection_database::get())
    }
}

impl PartialEq for Database {
    fn eq(&self, _other: &Self) -> bool {
        true // All database userdatas refer to the same underlying rbx-dom database
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReflectionDatabase")
    }
}
