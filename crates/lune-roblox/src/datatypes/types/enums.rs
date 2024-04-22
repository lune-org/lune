use core::fmt;

use mlua::prelude::*;

use super::{super::*, Enum};

/**
    An implementation of the [Enums](https://create.roblox.com/docs/reference/engine/datatypes/Enums) Roblox datatype.

    This implements all documented properties, methods & constructors of the Enums class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Enums;

impl LuaUserData for Enums {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Methods
        methods.add_method("GetEnums", |_, _, ()| {
            let db = rbx_reflection_database::get();
            Ok(db.enums.values().map(Enum::from).collect::<Vec<_>>())
        });
        methods.add_meta_method(
            LuaMetaMethod::Index,
            |_, _, name: String| match Enum::from_name(&name) {
                Some(e) => Ok(e),
                None => Err(LuaError::RuntimeError(format!(
                    "The enum '{name}' does not exist",
                ))),
            },
        );
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for Enums {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Enum")
    }
}
