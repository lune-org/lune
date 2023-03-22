use mlua::prelude::*;

use crate::shared::classes::add_class_restricted_method;

use super::Instance;

pub const CLASS_NAME: &str = "DataModel";

pub fn add_methods<'lua, M: LuaUserDataMethods<'lua, Instance>>(methods: &mut M) {
    add_class_restricted_method(
        methods,
        CLASS_NAME,
        "GetService",
        |_, _, _service_name: String| Ok(()),
    );
    add_class_restricted_method(
        methods,
        CLASS_NAME,
        "FindService",
        |_, _, _service_name: String| Ok(()),
    );
}
