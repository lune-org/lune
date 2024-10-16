use std::ptr;

use libffi::middle::{Cif, Type};
use mlua::prelude::*;

use super::{
    association_names::{CALLABLE_CFN, CALLABLE_REF, CFN_ARGS, CFN_RESULT},
    c_helper, method_provider,
};
use crate::ffi::{
    bit_mask::u8_test_not,
    ffi_association::{get_association, set_association},
    FfiCallable, FfiRef, FfiRefFlag, NativeArgInfo, NativeData, NativeResultInfo, NativeSignedness,
    NativeSize,
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

pub struct CFunc {
    cif: Cif,
    arg_info_list: Vec<NativeArgInfo>,
    result_info: NativeResultInfo,
}

impl NativeSignedness for CFunc {
    fn get_signedness(&self) -> bool {
        false
    }
}
impl NativeSize for CFunc {
    fn get_size(&self) -> usize {
        size_of::<*mut ()>()
    }
}

impl CFunc {
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

    pub fn new_from_table<'lua>(
        lua: &'lua Lua,
        arg_table: LuaTable,
        ret: LuaAnyUserData,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let args_types = c_helper::get_middle_type_list(&arg_table)?;
        let ret_type = c_helper::get_middle_type(&ret)?;

        let arg_len = arg_table.raw_len();
        let mut arg_info_list = Vec::<NativeArgInfo>::with_capacity(arg_len);
        for index in 0..arg_len {
            let userdata = c_helper::get_userdata(arg_table.raw_get(index + 1)?)?;
            arg_info_list.push(NativeArgInfo {
                conv: unsafe { c_helper::get_conv(&userdata)? },
                size: c_helper::get_size(&userdata)?,
            });
        }
        let result_info = NativeResultInfo {
            conv: unsafe { c_helper::get_conv(&ret)? },
            size: c_helper::get_size(&ret)?,
        };

        let cfn =
            lua.create_userdata(Self::new(args_types, ret_type, arg_info_list, result_info)?)?;

        // Create association to hold argument and result type
        set_association(lua, CFN_ARGS, &cfn, arg_table)?;
        set_association(lua, CFN_RESULT, &cfn, ret)?;

        Ok(cfn)
    }

    // Stringify for pretty printing like:
    // <CFunc( (u8, i32) -> u8 )>
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        let mut result = String::from(" (");
        if let (Some(LuaValue::Table(arg_table)), Some(LuaValue::UserData(result_userdata))) = (
            get_association(lua, CFN_ARGS, userdata)?,
            get_association(lua, CFN_RESULT, userdata)?,
        ) {
            let len = arg_table.raw_len();
            for arg_index in 1..=len {
                let arg_userdata: LuaAnyUserData = arg_table.raw_get(arg_index)?;
                let pretty_formatted = c_helper::pretty_format(lua, &arg_userdata)?;
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
                format!(") -> {} ", c_helper::pretty_format(lua, &result_userdata)?,).as_str(),
            );
            Ok(result)
        } else {
            Err(LuaError::external("failed to get inner type userdata."))
        }
    }
}

impl LuaUserData for CFunc {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Subtype
        method_provider::provide_ptr(methods);
        method_provider::provide_arr(methods);

        // ToString
        method_provider::provide_to_string(methods);

        // Realize
        // methods.add_method("closure", |lua, this, func: LuaFunction| {
        //     lua.create_userdata(FfiClosure::new(this.cif, userdata))
        // })
        methods.add_function(
            "callable",
            |lua, (cfn, function_ref): (LuaAnyUserData, LuaAnyUserData)| {
                let this = cfn.borrow::<CFunc>()?;

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
