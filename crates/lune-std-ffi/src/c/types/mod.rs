#![allow(clippy::inline_always)]

use core::ffi::*;
use std::cell::Ref;
use std::{any::TypeId, ops::Deref};

use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::{CType, CTypeCast};
use crate::ffi::{NativeConvert, NativeDataHandle, NativeSignedness};

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

macro_rules! cast_nums {
    ($T:ident, $self:ident, $from_ctype:ident, $into_ctype:ident, $from:ident, $into:ident, $t:ty, $($c:ty),*) => {
        $self
            .try_cast_num::<$T, $t>($into_ctype, $from, $into)?
            $(.or($self.try_cast_num::<$T, $c>($into_ctype, $from, $into)?))*
            .ok_or_else(|| $self.cast_failed_with($from_ctype, $into_ctype))
    };
}
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
        from: &Ref<dyn NativeDataHandle>,
        into: &Ref<dyn NativeDataHandle>,
    ) -> LuaResult<()> {
        cast_nums!(
            T, self, into_ctype, from_ctype, from, into, u8, u16, u32, u64, u128, i8, i16, i128,
            f32, f64, usize, isize
        )
    }
}

// export all default c-types
macro_rules! define_c_types {
    ( $lua:ident, $n:expr, $t:ident ) => {
        (
            $n,
            CType::<$t>::new_with_libffi_type($lua, Type::$t(), Some($n))?,
        )
    };
}
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
                Some("char"),
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
        define_c_types!(lua, "uchar", c_uchar),
        define_c_types!(lua, "schar", c_schar),
        define_c_types!(lua, "short", c_short),
        define_c_types!(lua, "ushort", c_ushort),
        define_c_types!(lua, "int", c_int),
        define_c_types!(lua, "uint", c_uint),
        define_c_types!(lua, "long", c_long),
        define_c_types!(lua, "ulong", c_ulong),
        define_c_types!(lua, "longlong", c_longlong),
        define_c_types!(lua, "ulonglong", c_ulonglong),
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

macro_rules! define_ctype_size_from_userdata {
    ($t:ident, $f:ty, $( $c:ty ),*) => {
        if $t.is::<CType<$f>>() {
            Ok(size_of::<$f>())
        }$( else if $t.is::<CType<$c>>() {
            Ok(size_of::<$c>())
        })* else {
            Err(LuaError::external("Unexpected type"))
        }
    };
}
#[inline(always)]
pub fn ctype_size_from_userdata(this: &LuaAnyUserData) -> LuaResult<usize> {
    define_ctype_size_from_userdata!(
        this, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64
    )
}

macro_rules! define_ctype_luavalue_into_ptr {
    ($lua:ident, $this:ident, $offset:ident, $data_handle:ident, $value:ident, $f:ty, $( $c:ty ),*) => {
        if $this.is::<CType<$f>>() {
            let ctype = $this.borrow::<CType<$f>>()?;
            ctype.luavalue_into($lua, $offset, $data_handle, $value)
        }$( else if $this.is::<CType<$c>>() {
            let ctype = $this.borrow::<CType<$c>>()?;
            ctype.luavalue_into($lua, $offset, $data_handle, $value)
        })* else {
            Err(LuaError::external("Unexpected type"))
        }
    };
}
#[inline(always)]
pub unsafe fn ctype_luavalue_into_ptr<'lua>(
    lua: &'lua Lua,
    this: &LuaAnyUserData<'lua>,
    offset: isize,
    data_handle: &Ref<dyn NativeDataHandle>,
    value: LuaValue<'lua>,
) -> LuaResult<()> {
    define_ctype_luavalue_into_ptr!(
        lua,
        this,
        offset,
        data_handle,
        value,
        u8,
        u16,
        u32,
        u64,
        u128,
        i8,
        i16,
        i32,
        i64,
        i128,
        f32,
        f64
    )
}

macro_rules! define_ctype_luavalue_from_ptr {
    ($lua:ident, $this:ident, $offset:ident, $data_handle:ident, $f:ty, $( $c:ty ),*) => {
        if $this.is::<CType<$f>>() {
            $this.borrow::<CType<$f>>()?.luavalue_from($lua, $offset, $data_handle)
        }$( else if $this.is::<CType<$c>>() {
            $this.borrow::<CType<$c>>()?.luavalue_from($lua, $offset, $data_handle)
        })* else {
            Err(LuaError::external("Unexpected type"))
        }
    };
}
#[inline(always)]
pub unsafe fn ctype_luavalue_from_ptr<'lua>(
    lua: &'lua Lua,
    this: &LuaAnyUserData<'lua>,
    offset: isize,
    data_handle: &Ref<dyn NativeDataHandle>,
) -> LuaResult<LuaValue<'lua>> {
    define_ctype_luavalue_from_ptr!(
        lua,
        this,
        offset,
        data_handle,
        u8,
        u16,
        u32,
        u64,
        u128,
        i8,
        i16,
        i32,
        i64,
        i128,
        f32,
        f64
    )
}

// struct CastCache<'a> {
//     conv: &'a [for<'lua> fn(lua: &'lua Lua)],
//     ud: Box<[*const dyn NativeConvert]>,
// }

// fn test<'a>(ud: &'a LuaAnyUserData) -> LuaResult<Box<CastCache<'a>>> {
// Box::new([(ud.to_pointer() as *const CType<u8>) as *const dyn NativeConvert])
// let ff: for<'lua> unsafe fn(
//     lua: &'lua Lua,
//     type_userdata: &LuaAnyUserData<'lua>,
//     offset: isize,
//     data_handle: &Ref<dyn NativeDataHandle>,
//     value: LuaValue<'lua>,
// ) -> LuaResult<()> = || CType::<f32>::luavalue_into;
// }

macro_rules! define_get_ctype_conv {
    ($userdata:ident, $f:ty, $( $c:ty ),*) => {
        if $userdata.is::<CType<$f>>() {
            Ok($userdata.to_pointer().cast::<CType<$f>>() as *const dyn NativeConvert)
        }$( else if $userdata.is::<CType<$c>>() {
            Ok($userdata.to_pointer().cast::<CType<$c>>() as *const dyn NativeConvert)
        })* else {
            Err(LuaError::external("Unexpected type"))
        }
    };
}
pub unsafe fn get_ctype_conv(userdata: &LuaAnyUserData) -> LuaResult<*const dyn NativeConvert> {
    define_get_ctype_conv!(userdata, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64)
}
