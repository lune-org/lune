mod c_arr;
mod c_func;
pub mod c_helper;
mod c_ptr;
mod c_string;
mod c_struct;
mod c_type;
mod types;

pub use self::{
    c_arr::CArr,
    c_func::CFunc,
    c_helper::method_provider,
    c_ptr::CPtr,
    c_struct::CStruct,
    c_type::{CType, CTypeCast},
    types::{c_type_helper, export_ctypes},
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
}
