#![allow(clippy::inline_always)]

use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::ReadWriteHandle;

pub trait NativeCast {
    // Cast T as U

    #[inline(always)]
    fn cast_num<T, U>(
        &self,
        from: impl ReadWriteHandle,
        into: impl ReadWriteHandle,
    ) -> LuaResult<()>
    where
        T: AsPrimitive<U>,
        U: 'static + Copy,
    {
        let from_ptr = unsafe { from.get_pointer(0)?.cast::<T>() };
        let into_ptr = unsafe { into.get_pointer(0)?.cast::<U>() };

        unsafe {
            *into_ptr = (*from_ptr).as_();
        }

        Ok(())
    }
}
