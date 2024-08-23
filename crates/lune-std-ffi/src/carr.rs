use libffi::middle::Type;
use mlua::prelude::*;

use crate::ctype::libffi_type_ensured_size;

// This is a series of some type.
// It provides the final size and the offset of the index,
// but does not allow multidimensional arrays because of API complexity.
// However, multidimensional arrays are not impossible to implement
// because they are a series of transcribed one-dimensional arrays.

// See: https://stackoverflow.com/a/43525176

// Padding after each field inside the struct is set to next field can follow the alignment.
// There is no problem even if you create a struct with n fields of a single type within the struct. Array adheres to the condition that there is no additional padding between each element. Padding to a struct is padding inside the struct. Simply think of the padding byte as a trailing unnamed field.

struct CArr {
    libffi_type: Type,
    struct_type: Type,
    length: usize,
    field_size: usize,
    size: usize,
}

impl CArr {
    fn new(libffi_type: Type, length: usize) -> LuaResult<Self> {
        let struct_type = Type::structure(vec![libffi_type.clone(); length]);
        let field_size = libffi_type_ensured_size(libffi_type.as_raw_ptr())?;

        Ok(Self {
            libffi_type,
            struct_type,
            length,
            field_size,
            size: field_size * length,
        })
    }
}

impl LuaUserData for CArr {}
