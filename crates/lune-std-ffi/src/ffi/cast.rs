use std::cell::Ref;

use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::FfiData;

#[inline]
pub fn num_cast<From, Into>(from: &Ref<dyn FfiData>, into: &Ref<dyn FfiData>) -> LuaResult<()>
where
    From: AsPrimitive<Into>,
    Into: 'static + Copy,
{
    let from_ptr = unsafe { from.get_inner_pointer().cast::<From>() };
    let into_ptr = unsafe { into.get_inner_pointer().cast::<Into>() };

    unsafe {
        *into_ptr = (*from_ptr).as_();
    }

    Ok(())
}
