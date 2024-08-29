pub mod ffi_association;
mod ffi_box;
pub mod ffi_helper;
mod ffi_lib;
mod ffi_native;
mod ffi_raw;
mod ffi_ref;

pub use self::{
    ffi_box::FfiBox,
    ffi_lib::FfiLib,
    ffi_native::{GetNativeDataHandle, NativeCast, NativeConvert, NativeDataHandle, NativeSized},
    ffi_ref::{create_nullptr, FfiRef},
};

// Named registry table names
mod association_names {
    pub const REF_INNER: &str = "__ref_inner";
}
