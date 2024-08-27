pub use types::create_all_c_types;
pub use types::create_all_types;

pub mod c_arr;
pub mod c_fn;
pub mod c_helper;
pub mod c_ptr;
pub mod c_string;
pub mod c_struct;
pub mod c_type;
pub mod types;

// Named registry table names
mod association_names {
    pub const CPTR_INNER: &str = "__cptr_inner";
    pub const CARR_INNER: &str = "__carr_inner";
    pub const CSTRUCT_INNER: &str = "__cstruct_inner";
    pub const CTYPE_STATIC: &str = "__ctype_static";
}
