use mlua::prelude::*;

use crate::instance::Instance;

use super::instance::class_is_a;

pub(crate) fn add_class_restricted_method<'lua, M: LuaUserDataMethods<'lua, Instance>, A, R, F>(
    methods: &mut M,
    class_name: &'static str,
    method_name: &'static str,
    method: F,
) where
    A: FromLuaMulti<'lua>,
    R: ToLuaMulti<'lua>,
    F: 'static + Fn(&'lua Lua, &Instance, A) -> LuaResult<R>,
{
    methods.add_method(method_name, move |lua, this, args| {
        if class_is_a(this.get_class_name(), class_name).unwrap_or(false) {
            method(lua, this, args)
        } else {
            Err(LuaError::RuntimeError(format!(
                "{} is not a valid member of {}",
                method_name, class_name
            )))
        }
    });
}

#[allow(dead_code)]
pub(crate) fn add_class_restricted_method_mut<
    'lua,
    M: LuaUserDataMethods<'lua, Instance>,
    A,
    R,
    F,
>(
    methods: &mut M,
    class_name: &'static str,
    method_name: &'static str,
    method: F,
) where
    A: FromLuaMulti<'lua>,
    R: ToLuaMulti<'lua>,
    F: 'static + Fn(&'lua Lua, &mut Instance, A) -> LuaResult<R>,
{
    methods.add_method_mut(method_name, move |lua, this, args| {
        if class_is_a(this.get_class_name(), class_name).unwrap_or(false) {
            method(lua, this, args)
        } else {
            Err(LuaError::RuntimeError(format!(
                "{} is not a valid member of {}",
                method_name, class_name
            )))
        }
    });
}
