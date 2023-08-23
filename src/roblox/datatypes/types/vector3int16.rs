use core::fmt;
use std::ops;

use glam::IVec3;
use mlua::prelude::*;
use rbx_dom_weak::types::Vector3int16 as DomVector3int16;

use crate::{lune::util::TableBuilder, roblox::exports::LuaExportsTable};

use super::super::*;

/**
    An implementation of the [Vector3int16](https://create.roblox.com/docs/reference/engine/datatypes/Vector3int16)
    Roblox datatype, backed by [`glam::IVec3`].

    This implements all documented properties, methods &
    constructors of the Vector3int16 class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3int16(pub IVec3);

impl LuaExportsTable<'_> for Vector3int16 {
    const EXPORT_NAME: &'static str = "Vector3int16";

    fn create_exports_table(lua: &Lua) -> LuaResult<LuaTable> {
        let vector3int16_new = |_, (x, y, z): (Option<i16>, Option<i16>, Option<i16>)| {
            Ok(Vector3int16(IVec3 {
                x: x.unwrap_or_default() as i32,
                y: y.unwrap_or_default() as i32,
                z: z.unwrap_or_default() as i32,
            }))
        };

        TableBuilder::new(lua)?
            .with_function("new", vector3int16_new)?
            .build_readonly()
    }
}

impl LuaUserData for Vector3int16 {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("X", |_, this| Ok(this.0.x));
        fields.add_field_method_get("Y", |_, this| Ok(this.0.y));
        fields.add_field_method_get("Z", |_, this| Ok(this.0.z));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Unm, userdata_impl_unm);
        methods.add_meta_method(LuaMetaMethod::Add, userdata_impl_add);
        methods.add_meta_method(LuaMetaMethod::Sub, userdata_impl_sub);
        methods.add_meta_method(LuaMetaMethod::Mul, userdata_impl_mul_i32);
        methods.add_meta_method(LuaMetaMethod::Div, userdata_impl_div_i32);
    }
}

impl fmt::Display for Vector3int16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.0.x, self.0.y)
    }
}

impl ops::Neg for Vector3int16 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vector3int16(-self.0)
    }
}

impl ops::Add for Vector3int16 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vector3int16(self.0 + rhs.0)
    }
}

impl ops::Sub for Vector3int16 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector3int16(self.0 - rhs.0)
    }
}

impl ops::Mul for Vector3int16 {
    type Output = Vector3int16;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl ops::Mul<i32> for Vector3int16 {
    type Output = Vector3int16;
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Div for Vector3int16 {
    type Output = Vector3int16;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl ops::Div<i32> for Vector3int16 {
    type Output = Vector3int16;
    fn div(self, rhs: i32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl From<DomVector3int16> for Vector3int16 {
    fn from(v: DomVector3int16) -> Self {
        Vector3int16(IVec3 {
            x: v.x.clamp(i16::MIN, i16::MAX) as i32,
            y: v.y.clamp(i16::MIN, i16::MAX) as i32,
            z: v.z.clamp(i16::MIN, i16::MAX) as i32,
        })
    }
}

impl From<Vector3int16> for DomVector3int16 {
    fn from(v: Vector3int16) -> Self {
        DomVector3int16 {
            x: v.0.x.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            y: v.0.y.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            z: v.0.z.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        }
    }
}
