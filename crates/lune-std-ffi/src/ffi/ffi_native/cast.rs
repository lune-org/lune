#![allow(clippy::inline_always)]

use std::cell::Ref;

use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::NativeData;

// Cast T as U

#[inline(always)]
pub fn native_num_cast<T, U>(
    from: &Ref<dyn NativeData>,
    into: &Ref<dyn NativeData>,
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
