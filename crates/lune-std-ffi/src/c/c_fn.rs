use std::ptr;

use libffi::middle::{Cif, Type};
use mlua::prelude::*;

use super::c_helper::{get_size, get_userdata};
use super::{
    association_names::{CALLABLE_CFN, CALLABLE_REF, CFN_ARGS, CFN_RESULT},
    c_helper::{get_conv, libffi_type_from_userdata, libffi_type_list_from_table},
};
use crate::ffi::{
    bit_mask::u8_test_not, ffi_association::set_association, FfiCallable, FfiRef, FfiRefFlag,
    NativeArgInfo, NativeData, NativeResultInfo,
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
    cif: Cif,
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
    ) -> LuaResult<Self> {
        // let cif = ;

        Ok(Self {
            cif: Cif::new(args.clone(), ret.clone()),
            arg_info_list,
            result_info,
        })
    }

    pub fn new_from_lua_table<'lua>(
        lua: &'lua Lua,
        arg_table: LuaTable,
        ret: LuaAnyUserData,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let args_types = libffi_type_list_from_table(lua, &arg_table)?;
        let ret_type = libffi_type_from_userdata(lua, &ret)?;

        let arg_len = arg_table.raw_len();
        let mut arg_info_list = Vec::<NativeArgInfo>::with_capacity(arg_len);
        for index in 0..arg_len {
            let userdata = get_userdata(arg_table.raw_get(index + 1)?)?;
            arg_info_list.push(NativeArgInfo {
                conv: unsafe { get_conv(&userdata)? },
                size: get_size(&userdata)?,
            });
        }
        let result_info = NativeResultInfo {
            conv: unsafe { get_conv(&ret)? },
            size: get_size(&ret)?,
        };

        let cfn =
            lua.create_userdata(Self::new(args_types, ret_type, arg_info_list, result_info)?)?;

        // Create association to hold argument and result type
        set_association(lua, CFN_ARGS, &cfn, arg_table)?;
        set_association(lua, CFN_RESULT, &cfn, ret)?;

        Ok(cfn)
    }
}

impl LuaUserData for CFn {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // methods.add_method("closure", |lua, this, func: LuaFunction| {
        //     lua.create_userdata(FfiClosure::new(this.cif, userdata))
        // })
        methods.add_function(
            "caller",
            |lua, (cfn, function_ref): (LuaAnyUserData, LuaAnyUserData)| {
                let this = cfn.borrow::<CFn>()?;

                if !function_ref.is::<FfiRef>() {
                    return Err(LuaError::external("argument 0 must be ffiref"));
                }

                let ffi_ref = function_ref.borrow::<FfiRef>()?;
                if u8_test_not(ffi_ref.flags, FfiRefFlag::Function.value()) {
                    return Err(LuaError::external("not a function ref"));
                }

                let callable = lua.create_userdata(unsafe {
                    FfiCallable::new(
                        this.cif.as_raw_ptr(),
                        ptr::from_ref(&this.arg_info_list),
                        ptr::from_ref(&this.result_info),
                        ffi_ref.get_pointer(0),
                    )
                })?;

                set_association(lua, CALLABLE_CFN, &callable, cfn.clone())?;
                set_association(lua, CALLABLE_REF, &callable, function_ref.clone())?;

                Ok(callable)
            },
        );
    }
}
