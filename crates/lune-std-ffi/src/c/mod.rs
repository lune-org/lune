use lune_utils::TableBuilder;
use mlua::prelude::*;

mod arr_info;
mod fn_info;
pub mod helper;
mod ptr_info;
mod struct_info;
mod type_info;
mod types;
mod void_info;

pub use self::{
    arr_info::CArrInfo,
    fn_info::CFnInfo,
    helper::method_provider,
    ptr_info::CPtrInfo,
    struct_info::CStructInfo,
    type_info::{CTypeCast, CTypeInfo},
    types::{ctype_helper, export_c_types, export_fixed_types},
    void_info::CVoidInfo,
};

// Named registry table names
mod association_names {
    pub const CPTR_INNER: &str = "__cptr_inner";
    pub const CARR_INNER: &str = "__carr_inner";
    pub const CSTRUCT_INNER: &str = "__cstruct_inner";
    pub const CFN_RESULT: &str = "__cfn_result";
    pub const CFN_ARGS: &str = "__cfn_args";
    pub const CALLABLE_REF: &str = "__callable_ref";
    pub const CALLABLE_CFN: &str = "__callable_cfn";
    pub const CLOSURE_FUNC: &str = "__closure_func";
    pub const CLOSURE_CFN: &str = "__closure_cfn";
}

pub fn export_c(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_value("void", CVoidInfo::new())?
        .with_values(export_c_types(lua)?)?
        .with_function("struct", |lua, types: LuaTable| {
            CStructInfo::from_table(lua, types)
        })?
        .with_function("fn", |lua, (args, ret): (LuaTable, LuaAnyUserData)| {
            CFnInfo::from_table(lua, args, ret)
        })?
        .build_readonly()
}
