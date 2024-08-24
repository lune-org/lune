pub(super) mod c_arr;
pub(super) mod c_fn;
pub(super) mod c_helper;
pub(super) mod c_ptr;
pub(super) mod c_string;
pub(super) mod c_struct;
pub(super) mod c_type;

// Named registry table names
mod association_names {
    pub const CPTR_INNER: &str = "__cptr_inner";
    pub const CARR_INNER: &str = "__carr_inner";
    pub const CSTRUCT_INNER: &str = "__cstruct_inner";
}
