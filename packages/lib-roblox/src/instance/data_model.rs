use mlua::prelude::*;

use crate::shared::classes::add_class_restricted_method;

use super::Instance;

pub fn add_methods<'lua, M: LuaUserDataMethods<'lua, Instance>>(methods: &mut M) {
    add_class_restricted_method(
        methods,
        "DataModel",
        "GetService",
        |_, _, _service_name: String| Ok(()),
    );
    add_class_restricted_method(
        methods,
        "DataModel",
        "FindService",
        |_, _, _service_name: String| Ok(()),
    );
}
