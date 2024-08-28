use core::ffi::*;
use std::any::TypeId;

use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::c_type::CType;
use super::c_type::CTypeCast;

pub mod f32;
pub mod f64;
pub mod i128;
pub mod i16;
pub mod i32;
pub mod i64;
pub mod i8;
pub mod isize;
pub mod u128;
pub mod u16;
pub mod u32;
pub mod u64;
pub mod u8;
pub mod usize;

impl<T> CTypeCast for CType<T>
where
    T: AsPrimitive<u8>
        + AsPrimitive<u16>
        + AsPrimitive<u32>
        + AsPrimitive<u64>
        + AsPrimitive<u128>
        + AsPrimitive<i8>
        + AsPrimitive<i16>
        + AsPrimitive<i32>
        + AsPrimitive<i64>
        + AsPrimitive<i128>
        + AsPrimitive<f32>
        + AsPrimitive<f64>
        + AsPrimitive<usize>
        + AsPrimitive<isize>,
{
    fn cast(
        &self,
        from_ctype: &LuaAnyUserData,
        into_ctype: &LuaAnyUserData,
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
    ) -> LuaResult<()> {
        self.try_cast_num::<T, u8>(into_ctype, from, into)?
            .or(self.try_cast_num::<T, u16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, u32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, u64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, u128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, i8>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, i16>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, i32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, i64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, i128>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, f32>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, f64>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, usize>(into_ctype, from, into)?)
            .or(self.try_cast_num::<T, isize>(into_ctype, from, into)?)
            .ok_or_else(|| self.cast_failed_with(from_ctype, into_ctype))
    }
}

// export all default c-types
pub fn create_all_c_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaAnyUserData)>> {
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

// export all default c-types
pub fn create_all_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaAnyUserData)>> {
    Ok(vec![
        self::u8::create_type(lua)?,
        self::u16::create_type(lua)?,
        self::u32::create_type(lua)?,
        self::u64::create_type(lua)?,
        self::u128::create_type(lua)?,
        self::i8::create_type(lua)?,
        self::i16::create_type(lua)?,
        self::i32::create_type(lua)?,
        self::i64::create_type(lua)?,
        self::i128::create_type(lua)?,
        self::f64::create_type(lua)?,
        self::f32::create_type(lua)?,
        self::usize::create_type(lua)?,
        self::isize::create_type(lua)?,
    ])
}
