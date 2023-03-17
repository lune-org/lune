use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::{
    NumberSequence as RbxNumberSequence, NumberSequenceKeypoint as RbxNumberSequenceKeypoint,
};

use super::{super::*, NumberSequenceKeypoint};

/**
    An implementation of the [NumberSequence](https://create.roblox.com/docs/reference/engine/datatypes/NumberSequence) Roblox datatype.

    This implements all documented properties, methods & constructors of the NumberSequence class as of March 2023.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct NumberSequence {
    pub(crate) keypoints: Vec<NumberSequenceKeypoint>,
}

impl NumberSequence {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        type ArgsColor = f32;
        type ArgsColors = (f32, f32);
        type ArgsKeypoints = Vec<NumberSequenceKeypoint>;
        datatype_table.set(
            "new",
            lua.create_function(|lua, args: LuaMultiValue| {
                if let Ok(value) = ArgsColor::from_lua_multi(args.clone(), lua) {
                    Ok(NumberSequence {
                        keypoints: vec![
                            NumberSequenceKeypoint {
                                time: 0.0,
                                value,
                                envelope: 0.0,
                            },
                            NumberSequenceKeypoint {
                                time: 1.0,
                                value,
                                envelope: 0.0,
                            },
                        ],
                    })
                } else if let Ok((v0, v1)) = ArgsColors::from_lua_multi(args.clone(), lua) {
                    Ok(NumberSequence {
                        keypoints: vec![
                            NumberSequenceKeypoint {
                                time: 0.0,
                                value: v0,
                                envelope: 0.0,
                            },
                            NumberSequenceKeypoint {
                                time: 1.0,
                                value: v1,
                                envelope: 0.0,
                            },
                        ],
                    })
                } else if let Ok(keypoints) = ArgsKeypoints::from_lua_multi(args, lua) {
                    Ok(NumberSequence { keypoints })
                } else {
                    // FUTURE: Better error message here using given arg types
                    Err(LuaError::RuntimeError(
                        "Invalid arguments to constructor".to_string(),
                    ))
                }
            })?,
        )
    }
}

impl LuaUserData for NumberSequence {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Keypoints", |_, this| Ok(this.keypoints.clone()));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for NumberSequence {
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

impl From<RbxNumberSequence> for NumberSequence {
    fn from(v: RbxNumberSequence) -> Self {
        Self {
            keypoints: v
                .keypoints
                .iter()
                .cloned()
                .map(NumberSequenceKeypoint::from)
                .collect(),
        }
    }
}

impl From<NumberSequence> for RbxNumberSequence {
    fn from(v: NumberSequence) -> Self {
        Self {
            keypoints: v
                .keypoints
                .iter()
                .cloned()
                .map(RbxNumberSequenceKeypoint::from)
                .collect(),
        }
    }
}
