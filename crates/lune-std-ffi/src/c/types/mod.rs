#![allow(clippy::inline_always)]

use core::ffi::*;
use std::{any::TypeId, cell::Ref};

use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::{CTypeCast, CTypeInfo};
use crate::ffi::{num_cast, FfiConvert, FfiData, FfiSize};

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
            CTypeInfo::<$rust_type>::from_middle_type($lua, $libffi_type, $name)?,
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
        $( if $into_ctype.is::<CTypeInfo<$into_rust_type>>() {
            num_cast::<$from_rust_type, $into_rust_type>($from, $into)
        } else )* {
            Err($self.cast_failed_with($from_ctype, $into_ctype))
        }
    };
}
impl<From> CTypeCast for CTypeInfo<From>
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
        from_info: &LuaAnyUserData,
        into_info: &LuaAnyUserData,
        from: &Ref<dyn FfiData>,
        into: &Ref<dyn FfiData>,
    ) -> LuaResult<()> {
        define_cast_num!(
            From, self, into_info, from_info, from, into,
            u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize
        )
    }
}

pub mod ctype_helper {
    use super::*;

    // To prevent drop NativeConvert, we must use ffi_association to ensure children keep alive
    macro_rules! define_get_conv {
        ($userdata:ident, $( $rust_type:ty )*) => {
            $( if $userdata.is::<CTypeInfo<$rust_type>>() {
                Ok($userdata.to_pointer().cast::<CTypeInfo<$rust_type>>() as *const dyn FfiConvert)
            } else )* {
                Err(LuaError::external("Unexpected type"))
            }
        };
    }
    #[inline]
    pub fn get_conv(userdata: &LuaAnyUserData) -> LuaResult<*const dyn FfiConvert> {
        define_get_conv!(userdata, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize)
    }

    // Get size of ctype (not includes struct, arr, ... only CType<*>)
    macro_rules! define_get_size {
        ($userdata:ident, $( $rust_type:ty )*) => {
            $( if $userdata.is::<CTypeInfo<$rust_type>>() {
                Ok($userdata.borrow::<CTypeInfo<$rust_type>>()?.get_size())
            } else )* {
                Err(LuaError::external("Unexpected type"))
            }
        };
    }
    #[inline]
    pub fn get_size(userdata: &LuaAnyUserData) -> LuaResult<usize> {
        define_get_size!(userdata, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize)
    }

    // Get name of ctype
    macro_rules! define_get_name {
        ($userdata:ident, $( $rust_type:ty )*) => {
            $( if $userdata.is::<CTypeInfo<$rust_type>>() {
                Ok(Some($userdata.borrow::<CTypeInfo<$rust_type>>()?.get_name()))
            } else )* {
                Ok(None)
            }
        };
    }
    #[inline]
    pub fn get_name(userdata: &LuaAnyUserData) -> LuaResult<Option<&'static str>> {
        define_get_name!(userdata, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize)
    }

    // Get libffi_type of ctype
    macro_rules! define_get_middle_type {
        ($userdata:ident, $( $rust_type:ty )*) => {
            $( if $userdata.is::<CTypeInfo<$rust_type>>() {
                Ok(Some($userdata.borrow::<CTypeInfo<$rust_type>>()?.get_type()))
            } else )* {
                Ok(None)
            }
        };
    }
    #[inline]
    pub fn get_middle_type(userdata: &LuaAnyUserData) -> LuaResult<Option<Type>> {
        define_get_middle_type!(userdata, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize)
    }

    macro_rules! define_is_ctype {
        ($userdata:ident, $( $rust_type:ty )*) => {
            $( if $userdata.is::<CTypeInfo<$rust_type>>() {
                true
            } else )* {
                false
            }
        };
    }
    #[inline]
    pub fn is_ctype(userdata: &LuaAnyUserData) -> bool {
        define_is_ctype!(userdata, u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 usize isize)
    }
}
