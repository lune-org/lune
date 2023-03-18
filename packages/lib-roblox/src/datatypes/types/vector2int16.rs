use core::fmt;
use std::ops;

use glam::IVec2;
use mlua::prelude::*;
use rbx_dom_weak::types::Vector2int16 as DomVector2int16;

use super::super::*;

/**
    An implementation of the [Vector2int16](https://create.roblox.com/docs/reference/engine/datatypes/Vector2int16)
    Roblox datatype, backed by [`glam::IVec2`].

    This implements all documented properties, methods &
    constructors of the Vector2int16 class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2int16(pub IVec2);

impl Vector2int16 {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        datatype_table.set(
            "new",
            lua.create_function(|_, (x, y): (Option<i16>, Option<i16>)| {
                Ok(Vector2int16(IVec2 {
                    x: x.unwrap_or_default() as i32,
                    y: y.unwrap_or_default() as i32,
                }))
            })?,
        )
    }
}

impl LuaUserData for Vector2int16 {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("X", |_, this| Ok(this.0.x));
        fields.add_field_method_get("Y", |_, this| Ok(this.0.y));
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

impl fmt::Display for Vector2int16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.0.x, self.0.y)
    }
}

impl ops::Neg for Vector2int16 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vector2int16(-self.0)
    }
}

impl ops::Add for Vector2int16 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vector2int16(self.0 + rhs.0)
    }
}

impl ops::Sub for Vector2int16 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2int16(self.0 - rhs.0)
    }
}

impl ops::Mul for Vector2int16 {
    type Output = Vector2int16;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl ops::Mul<i32> for Vector2int16 {
    type Output = Vector2int16;
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Div for Vector2int16 {
    type Output = Vector2int16;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl ops::Div<i32> for Vector2int16 {
    type Output = Vector2int16;
    fn div(self, rhs: i32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl From<DomVector2int16> for Vector2int16 {
    fn from(v: DomVector2int16) -> Self {
        Vector2int16(IVec2 {
            x: v.x.clamp(i16::MIN, i16::MAX) as i32,
            y: v.y.clamp(i16::MIN, i16::MAX) as i32,
        })
    }
}

impl From<Vector2int16> for DomVector2int16 {
    fn from(v: Vector2int16) -> Self {
        DomVector2int16 {
            x: v.0.x.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            y: v.0.y.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        }
    }
}
