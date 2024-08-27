use core::ffi::*;
use std::any::TypeId;

use libffi::middle::Type;
use mlua::prelude::*;

use super::c_type::CType;

mod f32;
mod f64;
mod i16;
mod i32;
mod i64;
mod i8;
mod u16;
mod u32;
mod u64;
mod u8;

// export all default c-types
pub fn create_all_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaAnyUserData)>> {
    Ok(vec![
        (
            "char",
            CType::<c_char>::new_with_libffi_type(
                lua,
                if TypeId::of::<c_char>() == TypeId::of::<u8>() {
                    Type::c_uchar()
                } else {
                    Type::c_schar()
                },
                Some("longlong"),
            )?,
        ),
        (
            "uchar",
            CType::<c_uchar>::new_with_libffi_type(lua, Type::c_uchar(), Some("uchar"))?,
        ),
        (
            "schar",
            CType::<c_schar>::new_with_libffi_type(lua, Type::c_schar(), Some("schar"))?,
        ),
        (
            "short",
            CType::<c_short>::new_with_libffi_type(lua, Type::c_short(), Some("short"))?,
        ),
        (
            "ushort",
            CType::<c_ushort>::new_with_libffi_type(lua, Type::c_ushort(), Some("ushort"))?,
        ),
        (
            "int",
            CType::<c_int>::new_with_libffi_type(lua, Type::c_int(), Some("int"))?,
        ),
        (
            "uint",
            CType::<c_uint>::new_with_libffi_type(lua, Type::c_uint(), Some("uint"))?,
        ),
        (
            "long",
            CType::<c_long>::new_with_libffi_type(lua, Type::c_long(), Some("long"))?,
        ),
        (
            "ulong",
            CType::<c_ulong>::new_with_libffi_type(lua, Type::c_ulong(), Some("ulong"))?,
        ),
        (
            "longlong",
            CType::<c_longlong>::new_with_libffi_type(lua, Type::c_longlong(), Some("longlong"))?,
        ),
        (
            "ulonglong",
            CType::<c_ulonglong>::new_with_libffi_type(
                lua,
                Type::c_ulonglong(),
                Some("ulonglong"),
            )?,
        ),
        (
            "float",
            CType::<c_float>::new_with_libffi_type(lua, Type::f32(), Some("float"))?,
        ),
        (
            "double",
            CType::<c_double>::new_with_libffi_type(lua, Type::f64(), Some("double"))?,
        ),
    ])
}
