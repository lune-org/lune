use core::fmt;
use std::ops;

use glam::{Vec2, Vec3};
use mlua::prelude::*;
use rbx_dom_weak::types::Vector2 as DomVector2;

use lune_utils::TableBuilder;

use crate::exports::LuaExportsTable;

use super::super::*;

/**
    An implementation of the [Vector2](https://create.roblox.com/docs/reference/engine/datatypes/Vector2)
    Roblox datatype, backed by [`glam::Vec2`].

    This implements all documented properties, methods &
    constructors of the Vector2 class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector2(pub Vec2);

impl LuaExportsTable<'_> for Vector2 {
    const EXPORT_NAME: &'static str = "Vector2";

    fn create_exports_table(lua: &Lua) -> LuaResult<LuaTable> {
        let vector2_new = |_, (x, y): (Option<f32>, Option<f32>)| {
            Ok(Vector2(Vec2 {
                x: x.unwrap_or_default(),
                y: y.unwrap_or_default(),
            }))
        };

        TableBuilder::new(lua)?
            .with_value("xAxis", Vector2(Vec2::X))?
            .with_value("yAxis", Vector2(Vec2::Y))?
            .with_value("zero", Vector2(Vec2::ZERO))?
            .with_value("one", Vector2(Vec2::ONE))?
            .with_function("new", vector2_new)?
            .build_readonly()
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
        methods.add_method("Angle", |_, this, rhs: LuaUserDataRef<Vector2>| {
            Ok(this.0.angle_between(rhs.0))
        });
        methods.add_method("Cross", |_, this, rhs: LuaUserDataRef<Vector2>| {
            let this_v3 = Vec3::new(this.0.x, this.0.y, 0f32);
            let rhs_v3 = Vec3::new(rhs.0.x, rhs.0.y, 0f32);
            Ok(this_v3.cross(rhs_v3).z)
        });
        methods.add_method("Dot", |_, this, rhs: LuaUserDataRef<Vector2>| {
            Ok(this.0.dot(rhs.0))
        });
        methods.add_method(
            "Lerp",
            |_, this, (rhs, alpha): (LuaUserDataRef<Vector2>, f32)| {
                Ok(Vector2(this.0.lerp(rhs.0, alpha)))
            },
        );
        methods.add_method("Max", |_, this, rhs: LuaUserDataRef<Vector2>| {
            Ok(Vector2(this.0.max(rhs.0)))
        });
        methods.add_method("Min", |_, this, rhs: LuaUserDataRef<Vector2>| {
            Ok(Vector2(this.0.min(rhs.0)))
        });
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Unm, userdata_impl_unm);
        methods.add_meta_method(LuaMetaMethod::Add, userdata_impl_add);
        methods.add_meta_method(LuaMetaMethod::Sub, userdata_impl_sub);
        methods.add_meta_method(LuaMetaMethod::Mul, userdata_impl_mul_f32);
        methods.add_meta_method(LuaMetaMethod::Div, userdata_impl_div_f32);
        methods.add_meta_method(LuaMetaMethod::IDiv, userdata_impl_idiv_f32);
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

impl ops::Mul for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl ops::Mul<f32> for Vector2 {
    type Output = Vector2;
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Div for Vector2 {
    type Output = Vector2;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl ops::Div<f32> for Vector2 {
    type Output = Vector2;
    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl IDiv for Vector2 {
    type Output = Vector2;
    fn idiv(self, rhs: Self) -> Self::Output {
        Self((self.0 / rhs.0).floor())
    }
}

impl IDiv<f32> for Vector2 {
    type Output = Vector2;
    fn idiv(self, rhs: f32) -> Self::Output {
        Self((self.0 / rhs).floor())
    }
}

impl From<DomVector2> for Vector2 {
    fn from(v: DomVector2) -> Self {
        Vector2(Vec2 { x: v.x, y: v.y })
    }
}

impl From<Vector2> for DomVector2 {
    fn from(v: Vector2) -> Self {
        DomVector2 { x: v.0.x, y: v.0.y }
    }
}
