#![allow(clippy::inline_always)]

use core::ffi::*;
use std::{any::TypeId, cell::Ref};

use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::{CType, CTypeCast};
use crate::ffi::{native_num_cast, NativeConvert, NativeData, NativeSize};

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

// create CType userdata and export
macro_rules! create_ctypes {
    ($lua:ident, $(( $name:expr, $rust_type:ty, $libffi_type:expr ),)* ) => {
        Ok(vec![$((
            $name,
            CType::<$rust_type>::new_with_libffi_type($lua, $libffi_type, Some($name))?,
        ),)*])
    };
}
pub fn export_ctypes(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaAnyUserData)>> {
    create_ctypes!(
        lua,
        // Export Compile-time known c-types
        ("char", c_char, {
            if TypeId::of::<c_char>() == TypeId::of::<u8>() {
                Type::c_uchar()
            } else {
                Type::c_schar()
            }
        }),
        ("uchar", c_uchar, Type::c_uchar()),
        ("schar", c_schar, Type::c_schar()),
        ("short", c_short, Type::c_short()),
        ("ushort", c_ushort, Type::c_ushort()),
        ("int", c_int, Type::c_int()),
        ("uint", c_uint, Type::c_uint()),
        ("long", c_long, Type::c_long()),
        ("ulong", c_ulong, Type::c_ulong()),
        ("longlong", c_longlong, Type::c_longlong()),
        ("ulonglong", c_ulonglong, Type::c_ulonglong()),
        // Export Source-time known c-types (fixed)
        ("u8", u8, Type::u8()),
        ("u16", u16, Type::u16()),
        ("u32", u32, Type::u32()),
        ("u64", u64, Type::u64()),
        ("u128", u128, Type::c_longlong()),
        ("i8", i8, Type::i8()),
        ("i16", i16, Type::i16()),
        ("i32", i32, Type::i32()),
        ("i64", i64, Type::i64()),
        ("i128", i128, Type::c_ulonglong()),
        ("f64", f64, Type::f64()),
        ("f32", f32, Type::f32()),
        ("usize", usize, Type::usize()),
        ("isize", isize, Type::isize()),
        // TODO: c_float and c_double sometime can be half and single,
        // TODO: but libffi-rs doesn't support it. need work-around or drop support
        ("float", f32, Type::f32()),
        ("double", f64, Type::f64()),
    )
}

// Implement type-casting for numeric ctypes
macro_rules! define_cast_num {
    ($from_rust_type:ident, $self:ident, $from_ctype:ident, $into_ctype:ident, $from:ident, $into:ident, $($into_rust_type:ty)*) => {
        $( if $into_ctype.is::<CType<$into_rust_type>>() {
            native_num_cast::<$from_rust_type, $into_rust_type>($from, $into)
        } else )* {
            Err($self.cast_failed_with($from_ctype, $into_ctype))
        }
    };
}
impl<From> CTypeCast for CType<From>
where
    From: AsPrimitive<u8>
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
        from: &Ref<dyn NativeData>,
        into: &Ref<dyn NativeData>,
    ) -> LuaResult<()> {
        define_cast_num!(
            From, self, into_ctype, from_ctype, from, into,
            u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize
        )
    }
}

// To prevent drop NativeConvert, we must use ffi_association to ensure children keep alive
macro_rules! define_get_conv {
    ($userdata:ident, $( $rust_type:ty )*) => {
        $( if $userdata.is::<CType<$rust_type>>() {
            Ok($userdata.to_pointer().cast::<CType<$rust_type>>() as *const dyn NativeConvert)
        } else )* {
            Err(LuaError::external("Unexpected type"))
        }
    };
}
pub fn get_ctype_conv(userdata: &LuaAnyUserData) -> LuaResult<*const dyn NativeConvert> {
    define_get_conv!(userdata, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize)
}

// Get size of ctype (not includes struct, arr, ... only CType<*>)
macro_rules! define_get_size {
    ($userdata:ident, $( $rust_type:ty )*) => {
        $( if $userdata.is::<CType<$rust_type>>() {
            Ok($userdata.borrow::<CType<$rust_type>>()?.get_size())
        } else )* {
            Err(LuaError::external("Unexpected type"))
        }
    };
}
pub fn get_ctype_size(userdata: &LuaAnyUserData) -> LuaResult<usize> {
    define_get_size!(userdata, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize)
}

// Get name of ctype
macro_rules! define_get_name {
    ($userdata:ident, $( $rust_type:ty )*) => {
        $( if $userdata.is::<CType<$rust_type>>() {
            Ok($userdata.borrow::<CType<$rust_type>>()?.stringify())
        } else )* {
            Err(LuaError::external("Unexpected type"))
        }
    };
}
pub fn get_ctype_name(userdata: &LuaAnyUserData) -> LuaResult<&str> {
    define_get_name!(userdata, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize)
}

// Get libffi_type of ctype
macro_rules! define_get_libffi_type {
    ($userdata:ident, $( $rust_type:ty )*) => {
        $( if $userdata.is::<CType<$rust_type>>() {
            Ok($userdata.borrow::<CType<$rust_type>>()?.get_size())
        } else )* {
            Err(LuaError::external("Unexpected type"))
        }
    };
}
pub fn get_ctype_libffi_type(userdata: &LuaAnyUserData) -> LuaResult<usize> {
    define_get_libffi_type!(userdata, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize)
}
