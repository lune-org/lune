
use mlua::prelude::*;
use num::cast::AsPrimitive;

use super::FfiData;

// Cast number type to another number type, with num::cast library
#[inline]
pub fn num_cast<From, Into>(
    from: &dyn FfiData,
    into: &dyn FfiData,
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

    // Offsets are caller-supplied, so the pointers may be unaligned
    unsafe {
        into_ptr.write_unaligned(from_ptr.read_unaligned().as_());
    }

    Ok(())
}
