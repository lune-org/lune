use libffi::middle::Type;
use mlua::prelude::*;

// This is a series of some type.
// It provides the final size and the offset of the index,
// but does not allow multidimensional arrays because of API complexity.
// However, multidimensional arrays are not impossible to implement
// because they are a series of transcribed one-dimensional arrays.

// See: https://stackoverflow.com/a/43525176

struct CArr {
    libffi_type: Type,
    length: usize,
    size: usize,
}

impl CArr {
    fn new(libffi_type: Type, length: usize) {
        Self { libffi_type }
    }
}

impl LuaUserData for CArr {}
