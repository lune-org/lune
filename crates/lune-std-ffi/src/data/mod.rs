use std::cell::Ref;

use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

mod box_data;
mod callable_data;
mod closure_data;
mod lib_data;
mod ref_data;

pub use crate::{
    data::{
        box_data::BoxData,
        callable_data::CallableData,
        closure_data::ClosureData,
        lib_data::LibData,
        ref_data::{create_nullptr, RefData, RefDataBounds, RefDataFlag},
    },
    ffi::{
        association, num_cast, FfiArgInfo, FfiConvert, FfiData, FfiResultInfo, FfiSignedness,
        FfiSize,
    },
};

// Named registry table names
mod association_names {
    pub const REF_INNER: &str = "__ref_inner";
    pub const SYM_INNER: &str = "__syn_inner";
}

pub trait GetFfiData {
    fn get_data_handle(&self) -> LuaResult<Ref<dyn FfiData>>;
}

impl GetFfiData for LuaAnyUserData<'_> {
    fn get_data_handle(&self) -> LuaResult<Ref<dyn FfiData>> {
        if self.is::<BoxData>() {
            Ok(self.borrow::<BoxData>()? as Ref<dyn FfiData>)
        } else if self.is::<RefData>() {
            Ok(self.borrow::<RefData>()? as Ref<dyn FfiData>)
        // } else if self.is::<FfiRaw>() {
        // Ok(self.borrow::<FfiRaw>()? as Ref<dyn ReadWriteHandle>)
        } else {
            let config = ValueFormatConfig::new();
            Err(LuaError::external(format!(
                "Expected FfiBox, FfiRef or FfiRaw. got {}",
                // what?
                pretty_format_value(&LuaValue::UserData(self.to_owned()), &config)
            )))
        }
    }
}
