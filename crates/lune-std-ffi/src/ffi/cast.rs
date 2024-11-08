use std::cell::Ref;

use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::FfiData;

// Cast number type to another number type, with num::cast library
#[inline]
pub fn num_cast<From, Into>(
    from: &Ref<dyn FfiData>,
    into: &Ref<dyn FfiData>,
    from_offset: isize,
    into_offset: isize,
) -> LuaResult<()>
where
    From: AsPrimitive<Into>,
    Into: 'static + Copy,
{
    let from_ptr = unsafe {
        from.get_inner_pointer()
            .byte_offset(from_offset)
            .cast::<From>()
    };
    let into_ptr = unsafe {
        into.get_inner_pointer()
            .byte_offset(into_offset)
            .cast::<Into>()
    };

    unsafe {
        *into_ptr = (*from_ptr).as_();
    }

    Ok(())
}
