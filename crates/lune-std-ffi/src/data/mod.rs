use std::cell::Ref;

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
    ref_data::{create_nullref, RefBounds, RefData, RefFlag},
};
use crate::ffi::FfiData;

// Named registry table names
mod association_names {
    pub const REF_INNER: &str = "__ref_inner";
    pub const SYM_INNER: &str = "__syn_inner";
    pub const CLSOURE_REF_INNER: &str = "__closure_ref_inner";
}

// Get dynamic FfiData handle from LuaValue and LuaAnyUserData
pub trait GetFfiData {
    fn get_ffi_data(&self) -> LuaResult<Ref<dyn FfiData>>;
    fn is_ffi_data(&self) -> bool;
}
impl GetFfiData for LuaAnyUserData<'_> {
    fn get_ffi_data(&self) -> LuaResult<Ref<dyn FfiData>> {
        if self.is::<BoxData>() {
            Ok(self.borrow::<BoxData>()? as Ref<dyn FfiData>)
        } else if self.is::<RefData>() {
            Ok(self.borrow::<RefData>()? as Ref<dyn FfiData>)
        } else if self.is::<ClosureData>() {
            Ok(self.borrow::<ClosureData>()? as Ref<dyn FfiData>)
        } else {
            let config = ValueFormatConfig::new();
            Err(LuaError::external(format!(
                "Expected FfiBox, FfiRef or ClosureData. got {}",
                pretty_format_value(&LuaValue::UserData(self.to_owned()), &config)
            )))
        }
    }
    fn is_ffi_data(&self) -> bool {
        self.is::<BoxData>() | self.is::<RefData>() | self.is::<ClosureData>()
    }
}
impl GetFfiData for LuaValue<'_> {
    fn get_ffi_data(&self) -> LuaResult<Ref<dyn FfiData>> {
        self.as_userdata()
            .ok_or_else(|| {
                let config = ValueFormatConfig::new();
                LuaError::external(format!(
                    "Expected FfiBox, FfiRef or ClosureData. got {}",
                    pretty_format_value(self, &config)
                ))
            })?
            .get_ffi_data()
    }
    fn is_ffi_data(&self) -> bool {
        self.as_userdata().map_or(false, GetFfiData::is_ffi_data)
    }
}
