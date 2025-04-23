use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::ColorSequenceKeypoint as DomColorSequenceKeypoint;

use lune_utils::TableBuilder;

use crate::exports::LuaExportsTable;

use super::{super::*, Color3};

/**
    An implementation of the [ColorSequenceKeypoint](https://create.roblox.com/docs/reference/engine/datatypes/ColorSequenceKeypoint) Roblox datatype.

    This implements all documented properties, methods & constructors of the `ColorSequenceKeypoint` class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorSequenceKeypoint {
    pub(crate) time: f32,
    pub(crate) color: Color3,
}

impl LuaExportsTable for ColorSequenceKeypoint {
    const EXPORT_NAME: &'static str = "ColorSequenceKeypoint";

    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        let color_sequence_keypoint_new =
            |_: &Lua, (time, color): (f32, LuaUserDataRef<Color3>)| {
                Ok(ColorSequenceKeypoint {
                    time,
                    color: *color,
                })
            };

        TableBuilder::new(lua)?
            .with_function("new", color_sequence_keypoint_new)?
            .build_readonly()
    }
}

impl LuaUserData for ColorSequenceKeypoint {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Time", |_, this| Ok(this.time));
        fields.add_field_method_get("Value", |_, this| Ok(this.color));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for ColorSequenceKeypoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} > {}", self.time, self.color)
    }
}

impl From<DomColorSequenceKeypoint> for ColorSequenceKeypoint {
    fn from(v: DomColorSequenceKeypoint) -> Self {
        Self {
            time: v.time,
            color: v.color.into(),
        }
    }
}

impl From<ColorSequenceKeypoint> for DomColorSequenceKeypoint {
    fn from(v: ColorSequenceKeypoint) -> Self {
        Self {
            time: v.time,
            color: v.color.into(),
        }
    }
}
