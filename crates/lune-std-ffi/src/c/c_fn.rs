use libffi::low::ffi_cif;
use libffi::middle::{Cif, Type};
use mlua::prelude::*;

use super::c_helper::{
    get_conv, get_conv_list_from_table, libffi_type_from_userdata, libffi_type_list_from_table,
};
use crate::ffi::{
    FfiClosure, NativeArgInfo, NativeArgType, NativeConvert, NativeResultInfo, NativeResultType,
};

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
    cif: *mut ffi_cif,
    arg_info_list: Vec<NativeArgInfo>,
    result_info: NativeResultInfo,
}

// support: Cfn as function pointer

impl CFn {
    pub fn new(
        args: Vec<Type>,
        ret: Type,
        arg_info_list: Vec<NativeArgInfo>,
        result_info: NativeResultInfo,
    ) -> Self {
        Self {
            cif: Cif::new(args.clone(), ret.clone()).as_raw_ptr(),
            arg_info_list,
            result_info,
        }
    }

    pub fn new_from_lua_table(lua: &Lua, args: LuaTable, ret: LuaAnyUserData) -> LuaResult<Self> {
        let args_types = libffi_type_list_from_table(lua, &args)?;
        let ret_type = libffi_type_from_userdata(lua, &ret)?;

        let len = args.raw_len();
        let mut arg_info_list = Vec::<NativeArgInfo>::with_capacity(len);

        for conv in unsafe { get_conv_list_from_table(&args)? } {
            arg_info_list.push(NativeArgInfo { conv })
        }

        // get_conv_list_from_table(&args)?.iter().map(|conv| {
        //     conv.to_owned()
        // }).collect()

        Ok(Self::new(args_types, ret_type, unsafe {}, unsafe {
            get_conv(&ret)?
        }))
    }
}

impl LuaUserData for CFn {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("closure", |lua, this, func: LuaFunction| {
            lua.create_userdata(FfiClosure::new(this.cif, userdata))
        })
    }
}
