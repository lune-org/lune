use mlua::prelude::*;

use super::{FfiData, GetFfiData};

pub mod method_provider {
    use super::*;

    // Implement copyFrom method
    pub fn provide_copy_from<Target, M>(methods: &mut M)
    where
        Target: FfiData + 'static,
        M: LuaUserDataMethods<Target>,
    {
        methods.add_function(
            "copyFrom",
            |_lua,
             (this_userdata, src, length, dst_offset, src_offset): (
                LuaAnyUserData,
                LuaAnyUserData,
                usize,
                Option<isize>,
                Option<isize>,
            )| unsafe {
                let this = this_userdata.borrow::<Target>()?;
                let dst_offset = dst_offset.unwrap_or(0);
                let src_offset = src_offset.unwrap_or(0);
                let src = src.get_ffi_data()?;

                if !src.is_readable() {
                    return Err(LuaError::external("Source is not readable"));
                }
                if !this.is_writable() {
                    return Err(LuaError::external("Destination is not writable"));
                }
                if !src.check_inner_boundary(src_offset, length) {
                    return Err(LuaError::external("Source out of bounds"));
                }
                if !this.check_inner_boundary(dst_offset, length) {
                    return Err(LuaError::external("Destination out of bounds"));
                }

                this.copy_from(&src, length, dst_offset, src_offset);

                Ok(this_userdata.clone())
            },
        );
    }

    // Implement readString method
    pub fn provide_read_string<Target, M>(methods: &mut M)
    where
        Target: FfiData + 'static,
        M: LuaUserDataMethods<Target>,
    {
        methods.add_method(
            "readString",
            |lua, this, (length, offset): (usize, Option<isize>)| unsafe {
                let offset = offset.unwrap_or(0);

                if !this.is_readable() {
                    return Err(LuaError::external("Source is not readable"));
                }
                if !this.check_inner_boundary(offset, length) {
                    return Err(LuaError::external("Source out of bounds"));
                }

                lua.create_string(this.read_string(length, offset))
            },
        );
    }

    // Implement readCString method
    pub fn provide_read_c_string<Target, M>(methods: &mut M)
    where
        Target: FfiData + 'static,
        M: LuaUserDataMethods<Target>,
    {
        methods.add_method("readCString", |lua, this, offset: Option<isize>| unsafe {
            let offset = offset.unwrap_or(0);

            if !this.is_readable() {
                return Err(LuaError::external("Source is not readable"));
            }

            // Scan for the null terminator, stopping at the boundary. Unsized
            // refs have no boundary and are scanned like a raw C string.
            let start = this.get_inner_pointer().cast::<u8>().byte_offset(offset);
            let mut length = 0;
            loop {
                if !this.check_inner_boundary(offset, length + 1) {
                    return Err(LuaError::external(
                        "No null terminator found within the readable boundary",
                    ));
                }
                if *start.add(length) == 0 {
                    break;
                }
                length += 1;
            }

            lua.create_string(this.read_string(length, offset))
        });
    }

    // Implement writeString method
    pub fn provide_write_string<Target, M>(methods: &mut M)
    where
        Target: FfiData + 'static,
        M: LuaUserDataMethods<Target>,
    {
        methods.add_function(
            "writeString",
            |_lua,
             (this_userdata, string, length, dst_offset, src_offset): (
                LuaAnyUserData,
                LuaString,
                Option<usize>,
                Option<isize>,
                Option<usize>,
            )| unsafe {
                let string_len = string.as_bytes().len();
                let dst_offset = dst_offset.unwrap_or(0);
                let src_offset = src_offset.unwrap_or(0);
                let length = match length {
                    Some(length) => length,
                    None => string_len.checked_sub(src_offset).ok_or_else(|| {
                        LuaError::external(format!(
                            "Source offset out of bounds (string length: {string_len}, got {src_offset})",
                        ))
                    })?,
                };
                let this = this_userdata.borrow::<Target>()?;

                // Source string boundary check
                let source_end = src_offset.checked_add(length);
                if source_end.is_none_or(|end| string_len < end) {
                    return Err(LuaError::external("Source out of bounds"));
                }

                if !this.is_writable() {
                    return Err(LuaError::external("Destination is not writable"));
                }
                if !this.check_inner_boundary(dst_offset, length) {
                    return Err(LuaError::external("Destination out of bounds"));
                }

                this.write_string(string, length, dst_offset, src_offset);
                Ok(this_userdata.clone())
            },
        );
    }

    // TODO: Should we add readBuffer/writeBuffer to move bytes to/from a Luau buffer, which
    // is mutable unlike a string? Only copy-based for now; using a buffer as
    // FFI memory directly needs mlua to expose the buffer's data pointer.

    // Bitwise ops are covered by Luau's bit32 (read an integer, operate, write
    // it back); only C bit-fields would need more. Base64 is covered by
    // @lune/serde paired with readString/writeString.
}
