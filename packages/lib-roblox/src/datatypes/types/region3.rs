use core::fmt;

use glam::{Mat4, Vec3};
use mlua::prelude::*;
use rbx_dom_weak::types::Region3 as DomRegion3;

use super::{super::*, CFrame, Vector3};

/**
    An implementation of the [Region3](https://create.roblox.com/docs/reference/engine/datatypes/Region3)
    Roblox datatype, backed by [`glam::Vec3`].

    This implements all documented properties, methods & constructors of the Region3 class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Region3 {
    pub(crate) min: Vec3,
    pub(crate) max: Vec3,
}

impl Region3 {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        datatype_table.set(
            "new",
            lua.create_function(|_, (min, max): (Vector3, Vector3)| {
                Ok(Region3 {
                    min: min.0,
                    max: max.0,
                })
            })?,
        )
    }
}

impl LuaUserData for Region3 {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("CFrame", |_, this| {
            Ok(CFrame(Mat4::from_translation(this.min.lerp(this.max, 0.5))))
        });
        fields.add_field_method_get("Size", |_, this| Ok(Vector3(this.max - this.min)));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Methods
        methods.add_method("ExpandToGrid", |_, this, resolution: f32| {
            Ok(Region3 {
                min: (this.min / resolution).floor() * resolution,
                max: (this.max / resolution).ceil() * resolution,
            })
        });
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for Region3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", Vector3(self.min), Vector3(self.max))
    }
}

impl From<DomRegion3> for Region3 {
    fn from(v: DomRegion3) -> Self {
        Region3 {
            min: Vector3::from(v.min).0,
            max: Vector3::from(v.max).0,
        }
    }
}

impl From<Region3> for DomRegion3 {
    fn from(v: Region3) -> Self {
        DomRegion3 {
            min: Vector3(v.min).into(),
            max: Vector3(v.max).into(),
        }
    }
}
