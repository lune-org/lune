use core::fmt;
use std::ops;

use glam::{Vec2, Vec3};
use mlua::prelude::*;
use rbx_dom_weak::types::Vector2 as RbxVector2;

use super::super::*;

/**
    An implementation of the [Vector2](https://create.roblox.com/docs/reference/engine/datatypes/Vector2)
    Roblox datatype, backed by [`glam::Vec2`].

    This implements all documented properties, methods &
    constructors of the Vector2 class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2(pub Vec2);

impl Vector2 {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        // Constants
        datatype_table.set("xAxis", Vector2(Vec2::X))?;
        datatype_table.set("yAxis", Vector2(Vec2::Y))?;
        datatype_table.set("zero", Vector2(Vec2::ZERO))?;
        datatype_table.set("one", Vector2(Vec2::ONE))?;
        // Constructors
        datatype_table.set(
            "new",
            lua.create_function(|_, (x, y): (Option<f32>, Option<f32>)| {
                Ok(Vector2(Vec2 {
                    x: x.unwrap_or_default(),
                    y: y.unwrap_or_default(),
                }))
            })?,
        )
    }
}

impl LuaUserData for Vector2 {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Magnitude", |_, this| Ok(this.0.length()));
        fields.add_field_method_get("Unit", |_, this| Ok(Vector2(this.0.normalize())));
        fields.add_field_method_get("X", |_, this| Ok(this.0.x));
        fields.add_field_method_get("Y", |_, this| Ok(this.0.y));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Methods
        methods.add_method("Cross", |_, this, rhs: Vector2| {
            let this_v3 = Vec3::new(this.0.x, this.0.y, 0f32);
            let rhs_v3 = Vec3::new(rhs.0.x, rhs.0.y, 0f32);
            Ok(this_v3.cross(rhs_v3).z)
        });
        methods.add_method("Dot", |_, this, rhs: Vector2| Ok(this.0.dot(rhs.0)));
        methods.add_method("Lerp", |_, this, (rhs, alpha): (Vector2, f32)| {
            Ok(Vector2(this.0.lerp(rhs.0, alpha)))
        });
        methods.add_method("Max", |_, this, rhs: Vector2| {
            Ok(Vector2(this.0.max(rhs.0)))
        });
        methods.add_method("Min", |_, this, rhs: Vector2| {
            Ok(Vector2(this.0.min(rhs.0)))
        });
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Unm, userdata_impl_unm);
        methods.add_meta_method(LuaMetaMethod::Add, userdata_impl_add);
        methods.add_meta_method(LuaMetaMethod::Sub, userdata_impl_sub);
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, rhs: LuaValue| {
            match &rhs {
                LuaValue::Number(n) => return Ok(Vector2(this.0 * Vec2::splat(*n as f32))),
                LuaValue::Integer(i) => return Ok(Vector2(this.0 * Vec2::splat(*i as f32))),
                LuaValue::UserData(ud) => {
                    if let Ok(vec) = ud.borrow::<Vector2>() {
                        return Ok(Vector2(this.0 * vec.0));
                    }
                }
                _ => {}
            };
            Err(LuaError::FromLuaConversionError {
                from: rhs.type_name(),
                to: "Vector2",
                message: Some(format!(
                    "Expected Vector2 or number, got {}",
                    rhs.type_name()
                )),
            })
        });
        methods.add_meta_method(LuaMetaMethod::Div, |_, this, rhs: LuaValue| {
            match &rhs {
                LuaValue::Number(n) => return Ok(Vector2(this.0 / Vec2::splat(*n as f32))),
                LuaValue::Integer(i) => return Ok(Vector2(this.0 / Vec2::splat(*i as f32))),
                LuaValue::UserData(ud) => {
                    if let Ok(vec) = ud.borrow::<Vector2>() {
                        return Ok(Vector2(this.0 / vec.0));
                    }
                }
                _ => {}
            };
            Err(LuaError::FromLuaConversionError {
                from: rhs.type_name(),
                to: "Vector2",
                message: Some(format!(
                    "Expected Vector2 or number, got {}",
                    rhs.type_name()
                )),
            })
        });
    }
}

impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.0.x, self.0.y)
    }
}

impl ops::Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vector2(-self.0)
    }
}

impl ops::Add for Vector2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vector2(self.0 + rhs.0)
    }
}

impl ops::Sub for Vector2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector2(self.0 - rhs.0)
    }
}

impl From<&RbxVector2> for Vector2 {
    fn from(v: &RbxVector2) -> Self {
        Vector2(Vec2 { x: v.x, y: v.y })
    }
}

impl From<&Vector2> for RbxVector2 {
    fn from(v: &Vector2) -> Self {
        RbxVector2 { x: v.0.x, y: v.0.y }
    }
}
