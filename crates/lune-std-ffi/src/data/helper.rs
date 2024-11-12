use mlua::prelude::*;

use super::{FfiData, GetFfiData};

pub mod method_provider {
    use super::*;

    // Implement copyFrom method
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
                    return Err(LuaError::external("Source out of bounds"));
                }
                if !this.check_inner_boundary(dst_offset, length) {
                    return Err(LuaError::external("Self out of bounds"));
                }

                this.copy_from(&src, length, dst_offset, src_offset);

                Ok(())
            },
        );
    }

    // Implement readString method
    pub fn provide_read_string<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiData,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "readString",
            |lua, this, (length, offset): (usize, Option<isize>)| unsafe {
                let offset = offset.unwrap_or(0);

                if !this.check_inner_boundary(offset, length) {
                    return Err(LuaError::external("Source out of bounds"));
                }

                lua.create_string(this.read_string(length, offset))
            },
        );
    }

    // Implement writeString method
    pub fn provide_write_string<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiData,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "writeString",
            |_lua,
             this,
             (string, length, dst_offset, src_offset): (
                LuaString,
                usize,
                Option<isize>,
                Option<usize>,
            )| unsafe {
                let dst_offset = dst_offset.unwrap_or(0);
                let src_offset = src_offset.unwrap_or(0);

                // Source string boundary check
                if string.as_bytes().len() < src_offset + length {
                    return Err(LuaError::external("Source out of bounds"));
                }

                // Self boundary check
                if !this.check_inner_boundary(dst_offset, length) {
                    return Err(LuaError::external("Self out of bounds"));
                }

                this.write_string(string, length, dst_offset, src_offset);
                Ok(())
            },
        );
    }

    // TODO: Bit operation support
    // TODO: writeBase64 and readBase64 methods
}
