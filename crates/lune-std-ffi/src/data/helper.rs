use mlua::prelude::*;

use super::{FfiData, GetFfiData};

pub mod method_provider {

    use super::*;

    pub fn provide_copy_from<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiData,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "copyFrom",
            |_lua,
             this,
             (src, length, dst_offset, src_offset): (
                LuaAnyUserData,
                usize,
                Option<isize>,
                Option<isize>,
            )| unsafe {
                let dst_offset = dst_offset.unwrap_or(0);
                let src_offset = src_offset.unwrap_or(0);
                let src = src.get_ffi_data()?;

                if !src.check_inner_boundary(src_offset, length) {
                    return Err(LuaError::external("Source boundary check failed"));
                }
                if !this.check_inner_boundary(dst_offset, length) {
                    return Err(LuaError::external("Self boundary check failed"));
                }

                this.copy_from(&src, length, dst_offset, src_offset);

                Ok(())
            },
        );
    }
}
