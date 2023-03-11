use std::ops;

use mlua::prelude::*;

pub(super) fn userdata_impl_to_string<D>(_: &Lua, datatype: &D, _: ()) -> LuaResult<String>
where
    D: LuaUserData + ToString + 'static,
{
    Ok(datatype.to_string())
}

pub(super) fn userdata_impl_eq<D>(_: &Lua, datatype: &D, value: LuaValue) -> LuaResult<bool>
where
    D: LuaUserData + PartialEq + 'static,
{
    if let LuaValue::UserData(ud) = value {
        if let Ok(value_as_datatype) = ud.borrow::<D>() {
            Ok(*datatype == *value_as_datatype)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

pub(super) fn userdata_impl_unm<D>(_: &Lua, datatype: &D, _: ()) -> LuaResult<D>
where
    D: LuaUserData + ops::Neg<Output = D> + Copy,
{
    Ok(-*datatype)
}

pub(super) fn userdata_impl_add<D>(_: &Lua, datatype: &D, value: D) -> LuaResult<D>
where
    D: LuaUserData + ops::Add<Output = D> + Copy,
{
    Ok(*datatype + value)
}

pub(super) fn userdata_impl_sub<D>(_: &Lua, datatype: &D, value: D) -> LuaResult<D>
where
    D: LuaUserData + ops::Sub<Output = D> + Copy,
{
    Ok(*datatype - value)
}
