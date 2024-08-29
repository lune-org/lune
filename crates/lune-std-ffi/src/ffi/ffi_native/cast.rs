#![allow(clippy::inline_always)]

use std::cell::Ref;

use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::NativeDataHandle;

pub trait NativeCast {
    // Cast T as U

    #[inline(always)]
    fn cast_num<T, U>(
        &self,
        from: &Ref<dyn NativeDataHandle>,
        into: &Ref<dyn NativeDataHandle>,
    ) -> LuaResult<()>
    where
        T: AsPrimitive<U>,
        U: 'static + Copy,
    {
        let from_ptr = unsafe { from.get_pointer(0).cast::<T>() };
        let into_ptr = unsafe { into.get_pointer(0).cast::<U>() };

        unsafe {
            *into_ptr = (*from_ptr).as_();
        }

        Ok(())
    }
}
