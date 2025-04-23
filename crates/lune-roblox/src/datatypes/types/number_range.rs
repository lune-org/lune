use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::NumberRange as DomNumberRange;

use lune_utils::TableBuilder;

use crate::exports::LuaExportsTable;

use super::super::*;

/**
    An implementation of the [NumberRange](https://create.roblox.com/docs/reference/engine/datatypes/NumberRange) Roblox datatype.

    This implements all documented properties, methods & constructors of the `NumberRange` class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberRange {
    pub(crate) min: f32,
    pub(crate) max: f32,
}

impl LuaExportsTable for NumberRange {
    const EXPORT_NAME: &'static str = "NumberRange";

    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        let number_range_new = |_: &Lua, (min, max): (f32, Option<f32>)| {
            Ok(match max {
                Some(max) => NumberRange {
                    min: min.min(max),
                    max: min.max(max),
                },
                None => NumberRange { min, max: min },
            })
        };

        TableBuilder::new(lua)?
            .with_function("new", number_range_new)?
            .build_readonly()
    }
}

impl LuaUserData for NumberRange {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Min", |_, this| Ok(this.min));
        fields.add_field_method_get("Max", |_, this| Ok(this.max));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for NumberRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.min, self.max)
    }
}

impl From<DomNumberRange> for NumberRange {
    fn from(v: DomNumberRange) -> Self {
        Self {
            min: v.min,
            max: v.max,
        }
    }
}

impl From<NumberRange> for DomNumberRange {
    fn from(v: NumberRange) -> Self {
        Self {
            min: v.min,
            max: v.max,
        }
    }
}
