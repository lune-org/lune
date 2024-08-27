#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::super::ffi_helper::get_ptr_from_userdata;

pub trait NativeCast {
    // Cast T as U
    fn cast_num<T, U>(&self, from: &LuaAnyUserData, into: &LuaAnyUserData) -> LuaResult<()>
    where
        T: AsPrimitive<U>,
        U: 'static + Copy,
    {
        let from_ptr = unsafe { get_ptr_from_userdata(from, None)?.cast::<T>() };
        let into_ptr = unsafe { get_ptr_from_userdata(into, None)?.cast::<U>() };

        unsafe {
            *into_ptr = (*from_ptr).as_();
        }

        Ok(())
    }
}
