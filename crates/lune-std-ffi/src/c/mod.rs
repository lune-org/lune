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
    types::{ctype_helper, export_ctypes},
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
