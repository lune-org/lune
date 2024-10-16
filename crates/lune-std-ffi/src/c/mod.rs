mod c_arr;
mod c_fn;
pub mod c_helper;
mod c_ptr;
mod c_string;
mod c_struct;
mod c_type;
mod types;

pub use self::{
    c_arr::CArr,
    c_fn::CFn,
    c_ptr::CPtr,
    c_struct::CStruct,
    c_type::{CType, CTypeCast, CTypeStatic},
};

pub use types::export_ctypes;

// Named registry table names
mod association_names {
    pub const CPTR_INNER: &str = "__cptr_inner";
    pub const CARR_INNER: &str = "__carr_inner";
    pub const CSTRUCT_INNER: &str = "__cstruct_inner";
    pub const CTYPE_STATIC: &str = "__ctype_static";
    pub const CFN_RESULT: &str = "__cfn_result";
    pub const CFN_ARGS: &str = "__cfn_args";
    pub const CALLABLE_REF: &str = "__callable_ref";
    pub const CALLABLE_CFN: &str = "__callable_cfn";
}
