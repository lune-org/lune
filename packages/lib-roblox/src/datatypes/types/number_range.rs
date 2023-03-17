use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::NumberRange as RbxNumberRange;

use super::super::*;

/**
    An implementation of the [NumberRange](https://create.roblox.com/docs/reference/engine/datatypes/NumberRange) Roblox datatype.

    This implements all documented properties, methods & constructors of the NumberRange class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberRange {
    pub(crate) min: f32,
    pub(crate) max: f32,
}

impl NumberRange {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        datatype_table.set(
            "new",
            lua.create_function(|_, (min, max): (f32, Option<f32>)| {
                Ok(match max {
                    Some(max) => NumberRange {
                        min: min.min(max),
                        max: min.max(max),
                    },
                    None => NumberRange { min, max: min },
                })
            })?,
        )
    }
}

impl LuaUserData for NumberRange {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Min", |_, this| Ok(this.min));
        fields.add_field_method_get("Max", |_, this| Ok(this.max));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for NumberRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.min, self.max)
    }
}

impl From<RbxNumberRange> for NumberRange {
    fn from(v: RbxNumberRange) -> Self {
        Self {
            min: v.min,
            max: v.max,
        }
    }
}

impl From<NumberRange> for RbxNumberRange {
    fn from(v: NumberRange) -> Self {
        Self {
            min: v.min,
            max: v.max,
        }
    }
}
