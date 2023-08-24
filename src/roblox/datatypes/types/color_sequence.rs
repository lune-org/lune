use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::{
    ColorSequence as DomColorSequence, ColorSequenceKeypoint as DomColorSequenceKeypoint,
};

use crate::{lune::util::TableBuilder, roblox::exports::LuaExportsTable};

use super::{super::*, Color3, ColorSequenceKeypoint};

/**
    An implementation of the [ColorSequence](https://create.roblox.com/docs/reference/engine/datatypes/ColorSequence) Roblox datatype.

    This implements all documented properties, methods & constructors of the ColorSequence class as of March 2023.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct ColorSequence {
    pub(crate) keypoints: Vec<ColorSequenceKeypoint>,
}

impl LuaExportsTable<'_> for ColorSequence {
    const EXPORT_NAME: &'static str = "ColorSequence";

    fn create_exports_table(lua: &Lua) -> LuaResult<LuaTable> {
        type ArgsColor<'lua> = LuaUserDataRef<'lua, Color3>;
        type ArgsColors<'lua> = (LuaUserDataRef<'lua, Color3>, LuaUserDataRef<'lua, Color3>);
        type ArgsKeypoints<'lua> = Vec<LuaUserDataRef<'lua, ColorSequenceKeypoint>>;

        let color_sequence_new = |lua, args: LuaMultiValue| {
            if let Ok(color) = ArgsColor::from_lua_multi(args.clone(), lua) {
                Ok(ColorSequence {
                    keypoints: vec![
                        ColorSequenceKeypoint {
                            time: 0.0,
                            color: *color,
                        },
                        ColorSequenceKeypoint {
                            time: 1.0,
                            color: *color,
                        },
                    ],
                })
            } else if let Ok((c0, c1)) = ArgsColors::from_lua_multi(args.clone(), lua) {
                Ok(ColorSequence {
                    keypoints: vec![
                        ColorSequenceKeypoint {
                            time: 0.0,
                            color: *c0,
                        },
                        ColorSequenceKeypoint {
                            time: 1.0,
                            color: *c1,
                        },
                    ],
                })
            } else if let Ok(keypoints) = ArgsKeypoints::from_lua_multi(args, lua) {
                Ok(ColorSequence {
                    keypoints: keypoints.iter().map(|k| **k).collect(),
                })
            } else {
                // FUTURE: Better error message here using given arg types
                Err(LuaError::RuntimeError(
                    "Invalid arguments to constructor".to_string(),
                ))
            }
        };

        TableBuilder::new(lua)?
            .with_function("new", color_sequence_new)?
            .build_readonly()
    }
}

impl LuaUserData for ColorSequence {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Keypoints", |_, this| Ok(this.keypoints.clone()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for ColorSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, keypoint) in self.keypoints.iter().enumerate() {
            if index < self.keypoints.len() - 1 {
                write!(f, "{}, ", keypoint)?;
            } else {
                write!(f, "{}", keypoint)?;
            }
        }
        Ok(())
    }
}

impl From<DomColorSequence> for ColorSequence {
    fn from(v: DomColorSequence) -> Self {
        Self {
            keypoints: v
                .keypoints
                .iter()
                .cloned()
                .map(ColorSequenceKeypoint::from)
                .collect(),
        }
    }
}

impl From<ColorSequence> for DomColorSequence {
    fn from(v: ColorSequence) -> Self {
        Self {
            keypoints: v
                .keypoints
                .iter()
                .cloned()
                .map(DomColorSequenceKeypoint::from)
                .collect(),
        }
    }
}
