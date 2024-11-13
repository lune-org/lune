#![allow(clippy::inline_always)]

use core::ffi::*;
use std::{any::TypeId, cell::Ref};

use libffi::middle::Type;
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::{CTypeCast, CTypeInfo};
use crate::ffi::{num_cast, FfiConvert, FfiData, FfiSize};

mod f32;
mod f64;
mod i128;
mod i16;
mod i32;
mod i64;
mod i8;
mod isize;
mod u128;
mod u16;
mod u32;
mod u64;
mod u8;
mod usize;

// CType userdata export
macro_rules! create_ctypes {
    ($lua:ident, $(( $name:expr, $rust_type:ty, $libffi_type:expr ),)* ) => {
        Ok(vec![$((
            $name,
            CTypeInfo::<$rust_type>::from_middle_type($lua, $libffi_type, $name)?,
        ),)*])
    };
}
pub fn export_c_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaAnyUserData)>> {
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
    )
}
pub fn export_fixed_types(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaAnyUserData)>> {
    create_ctypes!(
        lua,
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
        ("f32", f32, Type::f32()),
        ("f64", f64, Type::f64()),
    )
}

// Implement type-casting for numeric ctypes
macro_rules! define_cast_num {
    ($from_rust_type:ident, $self:ident, $from_ctype:ident, $into_ctype:ident, $from:ident, $into:ident, $fromOffset:ident, $intoOffset:ident, $($into_rust_type:ty)*) => {
        $( if $into_ctype.is::<CTypeInfo<$into_rust_type>>() {
            num_cast::<$from_rust_type, $into_rust_type>($from, $into, $fromOffset, $intoOffset)
        } else )* {
            Err($self.cast_failed_with($from_ctype, $into_ctype))
        }
    };
}
impl<From> CTypeCast for CTypeInfo<From>
where
    From: AsPrimitive<f32>
        + AsPrimitive<f64>
        + AsPrimitive<i128>
        + AsPrimitive<i16>
        + AsPrimitive<i32>
        + AsPrimitive<i64>
        + AsPrimitive<i8>
        + AsPrimitive<isize>
        + AsPrimitive<u128>
        + AsPrimitive<u16>
        + AsPrimitive<u32>
        + AsPrimitive<u64>
        + AsPrimitive<u8>
        + AsPrimitive<usize>,
{
    fn cast(
        &self,
        from_info: &LuaAnyUserData,
        into_info: &LuaAnyUserData,
        from: &Ref<dyn FfiData>,
        into: &Ref<dyn FfiData>,
        from_offset: isize,
        into_offset: isize,
    ) -> LuaResult<()> {
        define_cast_num!(
            From, self, from_info, into_info, from, into, from_offset, into_offset,
            f32 f64 i128 i16 i32 i64 i8 isize u128 u16 u32 u64 u8 usize
        )
    }
}

pub mod ctype_helper {
    use super::*;

    // To prevent droping NativeConvert, need to ensure userdata keep alive
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
        define_get_conv!(userdata, f32 f64 i128 i16 i32 i64 i8 isize u128 u16 u32 u64 u8 usize)
    }

    // Get libffi_type of ctype
    macro_rules! define_get_middle_type {
        ($userdata:ident, $( $rust_type:ty )*) => {
            $( if $userdata.is::<CTypeInfo<$rust_type>>() {
                Ok(Some($userdata.borrow::<CTypeInfo<$rust_type>>()?.get_middle_type()))
            } else )* {
                Ok(None)
            }
        };
    }
    #[inline]
    pub fn get_middle_type(userdata: &LuaAnyUserData) -> LuaResult<Option<Type>> {
        define_get_middle_type!(userdata, f32 f64 i128 i16 i32 i64 i8 isize u128 u16 u32 u64 u8 usize)
    }

    // Get size of ctype (not including struct, arr, ... only CType<*>)
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
        define_get_size!(userdata, f32 f64 i128 i16 i32 i64 i8 isize u128 u16 u32 u64 u8 usize)
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
        define_get_name!(userdata, f32 f64 i128 i16 i32 i64 i8 isize u128 u16 u32 u64 u8 usize)
    }

    // Check whether userdata is ctype or not
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
        define_is_ctype!(userdata, f32 f64 i128 i16 i32 i64 i8 isize u128 u16 u32 u64 u8 usize)
    }
}
