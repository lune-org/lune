#![allow(clippy::inline_always)]

use core::ffi::*;
use std::any::TypeId;

use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::c_type::CType;
use super::c_type::CTypeCast;
use crate::ffi::ffi_native::NativeConvert;

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
        from: &LuaAnyUserData,
        into: &LuaAnyUserData,
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
    ($this:ident, $lua:ident, $value:ident, $ptr:ident, $f:ty, $( $c:ty ),*) => {
        if $this.is::<CType<$f>>() {
            let ctype = $this.borrow::<CType<$f>>()?;
            ctype.luavalue_into_ptr($this, $lua, $value, $ptr)
        }$( else if $this.is::<CType<$c>>() {
            let ctype = $this.borrow::<CType<$c>>()?;
            ctype.luavalue_into_ptr($this, $lua, $value, $ptr)
        })* else {
            Err(LuaError::external("Unexpected type"))
        }
    };
}
#[inline(always)]
pub fn ctype_luavalue_into_ptr<'lua>(
    this: &LuaAnyUserData<'lua>,
    lua: &'lua Lua,
    value: LuaValue<'lua>,
    ptr: *mut (),
) -> LuaResult<()> {
    define_ctype_luavalue_into_ptr!(
        this, lua, value, ptr, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64
    )
}

macro_rules! define_ctype_luavalue_from_ptr {
    ($this:ident, $lua:ident, $ptr:ident, $f:ty, $( $c:ty ),*) => {
        if $this.is::<CType<$f>>() {
            let ctype = $this.borrow::<CType<$f>>()?;
            ctype.luavalue_from_ptr($this, $lua, $ptr)
        }$( else if $this.is::<CType<$c>>() {
            let ctype = $this.borrow::<CType<$c>>()?;
            ctype.luavalue_from_ptr($this, $lua, $ptr)
        })* else {
            Err(LuaError::external("Unexpected type"))
        }
    };
}
#[inline(always)]
pub fn ctype_luavalue_from_ptr<'lua>(
    this: &LuaAnyUserData<'lua>,
    lua: &'lua Lua,
    ptr: *mut (),
) -> LuaResult<LuaValue<'lua>> {
    define_ctype_luavalue_from_ptr!(
        this, lua, ptr, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64
    )
}
