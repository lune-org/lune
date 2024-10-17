use std::ptr;

use libffi::middle::{Cif, Type};
use mlua::prelude::*;

use super::{
    association_names::{
        CALLABLE_CFN, CALLABLE_REF, CFN_ARGS, CFN_RESULT, CLOSURE_CFN, CLOSURE_FUNC,
    },
    ctype_helper::is_ctype,
    helper, method_provider, CArrInfo, CPtrInfo, CStructInfo,
};
use crate::{
    data::{CallableData, ClosureData, RefData, RefFlag},
    ffi::{association, bit_mask::*, FfiArg, FfiData, FfiResult, FfiSignedness, FfiSize},
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
    arg_info_list: Vec<FfiArg>,
    result_info: FfiResult,
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

const CALLBACK_ARG_REF_FLAG_TYPE: u8 = RefFlag::Readable.value();
const CALLBACK_ARG_REF_FLAG_PTR: u8 = RefFlag::Dereferenceable.value() | RefFlag::Readable.value();
const CALLBACK_ARG_REF_FLAG_ARR: u8 = RefFlag::Readable.value() | RefFlag::Offsetable.value();
const CALLBACK_ARG_REF_FLAG_STRUCT: u8 = RefFlag::Readable.value() | RefFlag::Offsetable.value();
const CALLBACK_ARG_REF_FLAG_CFN: u8 = RefFlag::Function.value();

fn create_arg_info(userdata: &LuaAnyUserData) -> LuaResult<FfiArg> {
    let callback_ref_flag = if is_ctype(userdata) {
        CALLBACK_ARG_REF_FLAG_TYPE
    } else if userdata.is::<CPtrInfo>() {
        CALLBACK_ARG_REF_FLAG_PTR
    } else if userdata.is::<CArrInfo>() {
        CALLBACK_ARG_REF_FLAG_ARR
    } else if userdata.is::<CStructInfo>() {
        CALLBACK_ARG_REF_FLAG_STRUCT
    } else if userdata.is::<CFnInfo>() {
        CALLBACK_ARG_REF_FLAG_CFN
    } else {
        return Err(LuaError::external("unexpected type userdata"));
    };
    Ok(FfiArg {
        size: helper::get_size(userdata)?,
        callback_ref_flag,
    })
}

impl CFnInfo {
    pub fn new(
        args: Vec<Type>,
        ret: Type,
        arg_info_list: Vec<FfiArg>,
        result_info: FfiResult,
    ) -> LuaResult<Self> {
        Ok(Self {
            cif: Cif::new(args.clone(), ret.clone()),
            arg_info_list,
            result_info,
        })
    }

    pub fn from_table<'lua>(
        lua: &'lua Lua,
        arg_table: LuaTable,
        ret: LuaAnyUserData,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let args_types = helper::get_middle_type_list(&arg_table)?;
        let ret_type = helper::get_middle_type(&ret)?;

        let arg_len = arg_table.raw_len();
        let mut arg_info_list = Vec::<FfiArg>::with_capacity(arg_len);
        for index in 0..arg_len {
            let userdata = helper::get_userdata(arg_table.raw_get(index + 1)?)?;
            arg_info_list.push(create_arg_info(&userdata)?);
        }
        let result_info = FfiResult {
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

    pub fn create_closure<'lua>(
        &self,
        lua: &'lua Lua,
        this: &LuaAnyUserData,
        lua_function: LuaFunction<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let closure = ClosureData::new(
            ptr::from_ref(lua),
            self.cif.as_raw_ptr(),
            self.arg_info_list.clone(),
            self.result_info.clone(),
            lua.create_registry_value(&lua_function)?,
        )?;
        let closure_userdata = lua.create_userdata(closure)?;

        association::set(lua, CLOSURE_CFN, &closure_userdata, this)?;
        association::set(lua, CLOSURE_FUNC, &closure_userdata, lua_function)?;

        Ok(closure_userdata)
    }

    pub fn create_callable<'lua>(
        &self,
        lua: &'lua Lua,
        this: &LuaAnyUserData,
        target_ref: &LuaAnyUserData,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        if !target_ref.is::<RefData>() {
            return Err(LuaError::external("argument 0 must be ffiref"));
        }

        let ffi_ref = target_ref.borrow::<RefData>()?;
        if u8_test_not(ffi_ref.flags, RefFlag::Function.value()) {
            return Err(LuaError::external("not a function ref"));
        }

        let callable = lua.create_userdata(unsafe {
            CallableData::new(
                self.cif.as_raw_ptr(),
                self.arg_info_list.clone(),
                self.result_info.clone(),
                ffi_ref.get_pointer(),
            )
        })?;

        association::set(lua, CALLABLE_CFN, &callable, this)?;
        association::set(lua, CALLABLE_REF, &callable, target_ref)?;

        Ok(callable)
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
        methods.add_function(
            "closure",
            |lua, (cfn, func): (LuaAnyUserData, LuaFunction)| {
                let this = cfn.borrow::<CFnInfo>()?;
                this.create_closure(lua, cfn.as_ref(), func)
            },
        );
        methods.add_function(
            "callable",
            |lua, (cfn, target): (LuaAnyUserData, LuaAnyUserData)| {
                let this = cfn.borrow::<CFnInfo>()?;
                this.create_callable(lua, cfn.as_ref(), &target)
            },
        );
    }
}
