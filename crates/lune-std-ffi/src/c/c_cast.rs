use std::marker::PhantomData;

use mlua::prelude::*;

use super::c_type::CType;
use crate::ffi::ffi_helper::get_ptr_from_userdata;

struct CCast<T, U: From<T>> {
    _from: PhantomData<CType<T>>,
    _into: PhantomData<CType<U>>,
}

impl<T, U> CCast<T, U>
where
    U: From<T>,
{
    fn new(from: CType<T>, into: CType<U>) -> Self {
        Self {
            _from: PhantomData,
            _into: PhantomData,
        }
    }
}

impl<T, U> LuaUserData for CCast<T, U>
where
    T: Copy + Sized,
    U: From<T> + Sized,
{
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method(
            "cast",
            |_, _this, (from, into): (LuaAnyUserData, LuaAnyUserData)| {
                let from_ptr = unsafe { get_ptr_from_userdata(&from, None)?.cast::<T>() };
                let into_ptr = unsafe { get_ptr_from_userdata(&into, None)?.cast::<U>() };

                unsafe {
                    *into_ptr = U::from(*from_ptr);
                }

                Ok(())
            },
        );
    }
}
