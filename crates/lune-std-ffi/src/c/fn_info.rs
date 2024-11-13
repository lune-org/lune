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
    ffi::{
        association, bit_field::*, libffi_helper::SIZE_OF_POINTER, FfiArg, FfiData, FfiResult,
        FfiSignedness, FfiSize,
    },
};

// Function pointer type
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
        SIZE_OF_POINTER
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
        return Err(LuaError::external("Unexpected argument type"));
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
        if helper::has_void(&arg_table)? {
            return Err(LuaError::external("Arguments can not include void type"));
        }

        let args_types = helper::get_middle_type_list(&arg_table)?;
        let ret_type = helper::get_middle_type(&ret)?;

        let arg_info_list = helper::create_list(&arg_table, create_arg_info)?;
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

    // Stringify for pretty-print
    // ex: <CFn( (u8, i32) -> u8 )>
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
            Err(LuaError::external("Failed to retrieve inner type"))
        }
    }

    // Create ClosureData with lua function
    pub fn create_closure<'lua>(
        &self,
        lua: &'lua Lua,
        this: &LuaAnyUserData,
        lua_function: LuaFunction<'lua>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let closure_data = ClosureData::alloc(
            lua,
            self.cif.as_raw_ptr(),
            self.arg_info_list.clone(),
            self.result_info.clone(),
            lua.create_registry_value(&lua_function)?,
        )?;

        association::set(lua, CLOSURE_CFN, &closure_data, this)?;
        association::set(lua, CLOSURE_FUNC, &closure_data, lua_function)?;

        Ok(closure_data)
    }

    // Create CallableData from RefData
    pub fn create_callable<'lua>(
        &self,
        lua: &'lua Lua,
        this: &LuaAnyUserData,
        target_ref: &LuaAnyUserData,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        if !target_ref.is::<RefData>() {
            return Err(LuaError::external("Argument 'functionRef' must be RefData"));
        }

        let ffi_ref = target_ref.borrow::<RefData>()?;
        if u8_test_not(ffi_ref.flags, RefFlag::Function.value()) {
            return Err(LuaError::external(
                "Argument 'functionRef' is not a valid function reference",
            ));
        }

        let callable = lua.create_userdata(unsafe {
            CallableData::new(
                self.cif.as_raw_ptr(),
                self.arg_info_list.clone(),
                self.result_info.clone(),
                ffi_ref.get_inner_pointer(),
            )
        })?;

        association::set(lua, CALLABLE_CFN, &callable, this)?;
        association::set(lua, CALLABLE_REF, &callable, target_ref)?;

        Ok(callable)
    }

    pub fn get_middle_type() -> Type {
        Type::pointer()
    }
}

impl LuaUserData for CFnInfo {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_lua, _this| Ok(SIZE_OF_POINTER));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
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
