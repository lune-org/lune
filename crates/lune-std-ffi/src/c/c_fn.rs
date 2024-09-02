use libffi::middle::{Cif, Type};
use mlua::prelude::*;

use super::c_helper::{
    get_conv, get_conv_list_from_table, libffi_type_from_userdata, libffi_type_list_from_table,
};
use crate::ffi::NativeConvert;

// cfn is a type declaration for a function.
// Basically, when calling an external function, this type declaration
// is referred to and type conversion is automatically assisted.

// However, in order to save on type conversion costs,
// users keep values ​​they will use continuously in a box and use them multiple times.
// Alternatively, if the types are the same,you can save the cost of creating
// a new space by directly passing FfiRaw,
// the result value of another function or the argument value of the callback.

// Defining cfn simply lists the function's actual argument positions and conversions.
// You must decide how to process the data in Lua.

// The name cfn is intentional. This is because any *c_void is
// moved to a Lua function or vice versa.

pub struct CFn {
    libffi_cif: Cif,
    args_conv: Vec<*const dyn NativeConvert>,
    ret_conv: *const dyn NativeConvert,
}

impl CFn {
    pub fn new(
        args: Vec<Type>,
        ret: Type,
        args_conv: Vec<*const dyn NativeConvert>,
        ret_conv: *const dyn NativeConvert,
    ) -> Self {
        let libffi_cif: Cif = Cif::new(args.clone(), ret.clone());
        Self {
            libffi_cif,
            args_conv,
            ret_conv,
        }
    }

    pub fn new_from_lua_table(lua: &Lua, args: LuaTable, ret: LuaAnyUserData) -> LuaResult<Self> {
        let args_type = libffi_type_list_from_table(lua, &args)?;
        let ret_type = libffi_type_from_userdata(lua, &ret)?;

        Ok(Self::new(
            args_type,
            ret_type,
            unsafe { get_conv_list_from_table(&args)? },
            unsafe { get_conv(&ret)? },
        ))
    }
}

impl LuaUserData for CFn {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // methods.add_method("from", | this,  |)
    }
}
