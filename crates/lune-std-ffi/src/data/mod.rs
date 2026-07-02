use std::ops::Deref;

use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

mod box_data;
mod callable_data;
mod closure_data;
mod helper;
mod lib_data;
mod ref_data;

pub use self::{
    box_data::BoxData,
    callable_data::CallableData,
    closure_data::ClosureData,
    lib_data::LibData,
    ref_data::{create_nullref, RefBounds, RefData, RefFlag, UNSIZED_BOUNDS},
};
use crate::ffi::FfiData;

// Named registry keys
mod association_names {
    pub const REF_INNER: &str = "__ref_inner";
    pub const SYM_INNER: &str = "__syn_inner";
}

// Borrowed FfiData handle, keeps the underlying userdata borrow alive
pub enum FfiDataRef {
    Box(LuaUserDataRef<BoxData>),
    Ref(LuaUserDataRef<RefData>),
    Closure(LuaUserDataRef<ClosureData>),
}
impl Deref for FfiDataRef {
    type Target = dyn FfiData;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Box(data) => &**data,
            Self::Ref(data) => &**data,
            Self::Closure(data) => &**data,
        }
    }
}
// Delegate FfiData so that `&FfiDataRef` unsize-coerces to `&dyn FfiData`
// at call sites (rustc does not deref-coerce to trait objects)
impl FfiData for FfiDataRef {
    fn check_inner_boundary(&self, offset: isize, size: usize) -> bool {
        (**self).check_inner_boundary(offset, size)
    }
    unsafe fn get_inner_pointer(&self) -> *mut () {
        (**self).get_inner_pointer()
    }
    fn is_writable(&self) -> bool {
        (**self).is_writable()
    }
    fn is_readable(&self) -> bool {
        (**self).is_readable()
    }
}

// Get dynamic FfiData handle from LuaValue and LuaAnyUserData
pub trait GetFfiData {
    fn get_ffi_data(&self) -> LuaResult<FfiDataRef>;
}
impl GetFfiData for LuaAnyUserData {
    fn get_ffi_data(&self) -> LuaResult<FfiDataRef> {
        if self.is::<BoxData>() {
            Ok(FfiDataRef::Box(self.borrow::<BoxData>()?))
        } else if self.is::<RefData>() {
            Ok(FfiDataRef::Ref(self.borrow::<RefData>()?))
        } else if self.is::<ClosureData>() {
            Ok(FfiDataRef::Closure(self.borrow::<ClosureData>()?))
        } else {
            let config = ValueFormatConfig::new();
            Err(LuaError::external(format!(
                "Expected a BoxData, RefData or ClosureData, got {}",
                pretty_format_value(&LuaValue::UserData(self.to_owned()), &config)
            )))
        }
    }
}
impl GetFfiData for LuaValue {
    fn get_ffi_data(&self) -> LuaResult<FfiDataRef> {
        self.as_userdata()
            .ok_or_else(|| {
                let config = ValueFormatConfig::new();
                LuaError::external(format!(
                    "Expected a BoxData, RefData or ClosureData, got {}",
                    pretty_format_value(self, &config)
                ))
            })?
            .get_ffi_data()
    }
}
