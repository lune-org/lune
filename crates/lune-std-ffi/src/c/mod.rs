pub(super) mod c_arr;
pub(super) mod c_fn;
pub(super) mod c_helper;
pub(super) mod c_ptr;
pub(super) mod c_string;
pub(super) mod c_struct;
pub(super) mod c_type;

// Named registry table names
mod association_names {
    pub const CPTR_INNER: &str = "__cptr_inner";
    pub const CARR_INNER: &str = "__carr_inner";
    pub const CSTRUCT_INNER: &str = "__cstruct_inner";
}

use core::ffi::{
    c_char, c_double, c_float, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint,
    c_ulong, c_ulonglong, c_ushort, c_void,
};

use libffi::middle::Type;
use mlua::prelude::*;

use self::c_type::CType;
use crate::ffi::ffi_platform::CHAR_IS_SIGNED;

// export all default c-types
#[allow(clippy::too_many_lines)]
pub fn create_all_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaValue)>> {
    Ok(vec![
        (
            "int",
            CType::new(
                Type::c_int(),
                Some(String::from("int")),
                |data, ptr| {
                    let value = match data {
                        LuaValue::Integer(t) => t,
                        _ => {
                            return Err(LuaError::external(format!(
                                "Integer expected, got {}",
                                data.type_name()
                            )))
                        }
                    } as c_int;
                    unsafe {
                        *(ptr.cast::<c_int>()) = value;
                    }
                    Ok(())
                },
                |lua: &Lua, ptr: *mut ()| {
                    let value = unsafe { (*ptr.cast::<c_int>()).into_lua(lua)? };
                    Ok(value)
                },
            )?
            .into_lua(lua)?,
        ),
        (
            "long",
            CType::new(
                Type::c_long(),
                Some(String::from("long")),
                |data, ptr| {
                    let value = match data {
                        LuaValue::Integer(t) => t,
                        _ => {
                            return Err(LuaError::external(format!(
                                "Integer expected, got {}",
                                data.type_name()
                            )))
                        }
                    } as c_long;
                    unsafe {
                        *(ptr.cast::<c_long>()) = value;
                    }
                    Ok(())
                },
                |lua: &Lua, ptr: *mut ()| {
                    let value = unsafe { (*ptr.cast::<c_long>()).into_lua(lua)? };
                    Ok(value)
                },
            )?
            .into_lua(lua)?,
        ),
        (
            "longlong",
            CType::new(
                Type::c_longlong(),
                Some(String::from("longlong")),
                |data, ptr| {
                    let value = match data {
                        LuaValue::Integer(t) => t,
                        _ => {
                            return Err(LuaError::external(format!(
                                "Integer expected, got {}",
                                data.type_name()
                            )))
                        }
                    } as c_longlong;
                    unsafe {
                        *(ptr.cast::<c_longlong>()) = value;
                    }
                    Ok(())
                },
                |lua: &Lua, ptr: *mut ()| {
                    let value = unsafe { (*ptr.cast::<c_longlong>()).into_lua(lua)? };
                    Ok(value)
                },
            )?
            .into_lua(lua)?,
        ),
        (
            "char",
            CType::new(
                if CHAR_IS_SIGNED {
                    Type::c_schar()
                } else {
                    Type::c_uchar()
                },
                Some(String::from("char")),
                |data, ptr| {
                    let value = match data {
                        LuaValue::Integer(t) => t,
                        _ => {
                            return Err(LuaError::external(format!(
                                "Integer expected, got {}",
                                data.type_name()
                            )))
                        }
                    } as c_char;
                    unsafe {
                        *(ptr.cast::<c_char>()) = value;
                    }
                    Ok(())
                },
                |lua: &Lua, ptr: *mut ()| {
                    let value = unsafe { (*ptr.cast::<c_char>()).into_lua(lua)? };
                    Ok(value)
                },
            )?
            .into_lua(lua)?,
        ),
    ])
}
