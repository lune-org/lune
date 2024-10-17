use std::ptr;

use libffi::middle::{Cif, Type};
use mlua::prelude::*;

use super::{
    association_names::{CALLABLE_CFN, CALLABLE_REF, CFN_ARGS, CFN_RESULT},
    helper, method_provider,
};
use crate::{
    data::{CallableData, RefData, RefDataFlag},
    ffi::{association, bit_mask::*, FfiArgInfo, FfiData, FfiResultInfo, FfiSignedness, FfiSize},
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

pub struct CFnInfo {
    cif: Cif,
    arg_info_list: Vec<FfiArgInfo>,
    result_info: FfiResultInfo,
}

impl FfiSignedness for CFnInfo {
    fn get_signedness(&self) -> bool {
        false
    }
}
impl FfiSize for CFnInfo {
    fn get_size(&self) -> usize {
        size_of::<*mut ()>()
    }
}

impl CFnInfo {
    pub fn new(
        args: Vec<Type>,
        ret: Type,
        arg_info_list: Vec<FfiArgInfo>,
        result_info: FfiResultInfo,
    ) -> LuaResult<Self> {
        // let cif = ;

        Ok(Self {
            cif: Cif::new(args.clone(), ret.clone()),
            arg_info_list,
            result_info,
        })
    }

    pub fn new_from_table<'lua>(
        lua: &'lua Lua,
        arg_table: LuaTable,
        ret: LuaAnyUserData,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let args_types = helper::get_middle_type_list(&arg_table)?;
        let ret_type = helper::get_middle_type(&ret)?;

        let arg_len = arg_table.raw_len();
        let mut arg_info_list = Vec::<FfiArgInfo>::with_capacity(arg_len);
        for index in 0..arg_len {
            let userdata = helper::get_userdata(arg_table.raw_get(index + 1)?)?;
            arg_info_list.push(FfiArgInfo {
                conv: unsafe { helper::get_conv(&userdata)? },
                size: helper::get_size(&userdata)?,
            });
        }
        let result_info = FfiResultInfo {
            conv: unsafe { helper::get_conv(&ret)? },
            size: helper::get_size(&ret)?,
        };

        let cfn =
            lua.create_userdata(Self::new(args_types, ret_type, arg_info_list, result_info)?)?;

        // Create association to hold argument and result type
        association::set(lua, CFN_ARGS, &cfn, arg_table)?;
        association::set(lua, CFN_RESULT, &cfn, ret)?;

        Ok(cfn)
    }

    // Stringify for pretty printing like:
    // <CFunc( (u8, i32) -> u8 )>
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        let mut result = String::from(" (");
        if let (Some(LuaValue::Table(arg_table)), Some(LuaValue::UserData(result_userdata))) = (
            association::get(lua, CFN_ARGS, userdata)?,
            association::get(lua, CFN_RESULT, userdata)?,
        ) {
            let len = arg_table.raw_len();
            for arg_index in 1..=len {
                let arg_userdata: LuaAnyUserData = arg_table.raw_get(arg_index)?;
                let pretty_formatted = helper::pretty_format(lua, &arg_userdata)?;
                result.push_str(
                    (if len == arg_index {
                        pretty_formatted
                    } else {
                        format!("{pretty_formatted}, ")
                    })
                    .as_str(),
                );
            }
            result.push_str(
                format!(") -> {} ", helper::pretty_format(lua, &result_userdata)?,).as_str(),
            );
            Ok(result)
        } else {
            Err(LuaError::external("failed to get inner type userdata."))
        }
    }
}

impl LuaUserData for CFnInfo {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Subtype
        method_provider::provide_ptr_info(methods);
        method_provider::provide_arr_info(methods);

        // ToString
        method_provider::provide_to_string(methods);

        // Realize
        // methods.add_method("closure", |lua, this, func: LuaFunction| {
        //     lua.create_userdata(FfiClosure::new(this.cif, userdata))
        // })
        methods.add_function(
            "callable",
            |lua, (cfn, function_ref): (LuaAnyUserData, LuaAnyUserData)| {
                let this = cfn.borrow::<CFnInfo>()?;

                if !function_ref.is::<RefData>() {
                    return Err(LuaError::external("argument 0 must be ffiref"));
                }

                let ffi_ref = function_ref.borrow::<RefData>()?;
                if u8_test_not(ffi_ref.flags, RefDataFlag::Function.value()) {
                    return Err(LuaError::external("not a function ref"));
                }

                let callable = lua.create_userdata(unsafe {
                    CallableData::new(
                        this.cif.as_raw_ptr(),
                        ptr::from_ref(&this.arg_info_list),
                        ptr::from_ref(&this.result_info),
                        ffi_ref.get_pointer(),
                    )
                })?;

                association::set(lua, CALLABLE_CFN, &callable, cfn.clone())?;
                association::set(lua, CALLABLE_REF, &callable, function_ref.clone())?;

                Ok(callable)
            },
        );
    }
}
