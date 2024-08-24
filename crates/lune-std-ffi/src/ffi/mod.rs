pub(super) mod ffi_association;
pub(super) mod ffi_box;
pub(super) mod ffi_helper;
pub(super) mod ffi_lib;
pub(super) mod ffi_platform;
pub(super) mod ffi_raw;
pub(super) mod ffi_ref;

// Named registry table names
mod association_names {
    pub const BOX_REF_INNER: &str = "__box_ref";
    pub const REF_INNER: &str = "__ref_inner";
}
