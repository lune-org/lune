use core::fmt;
use std::ops;

use glam::Vec3;
use mlua::prelude::*;
use rbx_dom_weak::types::Vector3 as DomVector3;

use lune_utils::TableBuilder;

use crate::{datatypes::util::round_float_decimal, exports::LuaExportsTable};

use super::{super::*, EnumItem};

/**
    An implementation of the [Vector3](https://create.roblox.com/docs/reference/engine/datatypes/Vector3)
    Roblox datatype, backed by [`glam::Vec3`].

    This implements all documented properties, methods &
    constructors of the Vector3 class as of March 2023.

    Note that this does not use native Luau vectors to simplify implementation
    and instead allow us to implement all abovementioned APIs accurately.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3(pub Vec3);

impl LuaExportsTable<'_> for Vector3 {
    const EXPORT_NAME: &'static str = "Vector3";

    fn create_exports_table(lua: &Lua) -> LuaResult<LuaTable> {
        let vector3_from_axis = |_, normal_id: LuaUserDataRef<EnumItem>| {
            if normal_id.parent.desc.name == "Axis" {
                Ok(match normal_id.name.as_str() {
                    "X" => Vector3(Vec3::X),
                    "Y" => Vector3(Vec3::Y),
                    "Z" => Vector3(Vec3::Z),
                    name => {
                        return Err(LuaError::RuntimeError(format!(
                            "Axis '{name}' is not known",
                        )))
                    }
                })
            } else {
                Err(LuaError::RuntimeError(format!(
                    "EnumItem must be a Axis, got {}",
                    normal_id.parent.desc.name
                )))
            }
        };

        let vector3_from_normal_id = |_, normal_id: LuaUserDataRef<EnumItem>| {
            if normal_id.parent.desc.name == "NormalId" {
                Ok(match normal_id.name.as_str() {
                    "Left" => Vector3(Vec3::X),
                    "Top" => Vector3(Vec3::Y),
                    "Front" => Vector3(-Vec3::Z),
                    "Right" => Vector3(-Vec3::X),
                    "Bottom" => Vector3(-Vec3::Y),
                    "Back" => Vector3(Vec3::Z),
                    name => {
                        return Err(LuaError::RuntimeError(format!(
                            "NormalId '{name}' is not known",
                        )))
                    }
                })
            } else {
                Err(LuaError::RuntimeError(format!(
                    "EnumItem must be a NormalId, got {}",
                    normal_id.parent.desc.name
                )))
            }
        };

        let vector3_new = |_, (x, y, z): (Option<f32>, Option<f32>, Option<f32>)| {
            Ok(Vector3(Vec3 {
                x: x.unwrap_or_default(),
                y: y.unwrap_or_default(),
                z: z.unwrap_or_default(),
            }))
        };

        TableBuilder::new(lua)?
            .with_value("xAxis", Vector3(Vec3::X))?
            .with_value("yAxis", Vector3(Vec3::Y))?
            .with_value("zAxis", Vector3(Vec3::Z))?
            .with_value("zero", Vector3(Vec3::ZERO))?
            .with_value("one", Vector3(Vec3::ONE))?
            .with_function("fromAxis", vector3_from_axis)?
            .with_function("fromNormalId", vector3_from_normal_id)?
            .with_function("new", vector3_new)?
            .build_readonly()
    }
}

impl LuaUserData for Vector3 {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Magnitude", |_, this| Ok(this.0.length()));
        fields.add_field_method_get("Unit", |_, this| Ok(Vector3(this.0.normalize())));
        fields.add_field_method_get("X", |_, this| Ok(this.0.x));
        fields.add_field_method_get("Y", |_, this| Ok(this.0.y));
        fields.add_field_method_get("Z", |_, this| Ok(this.0.z));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Methods
        methods.add_method("Angle", |_, this, rhs: LuaUserDataRef<Vector3>| {
            Ok(this.0.angle_between(rhs.0))
        });
        methods.add_method("Cross", |_, this, rhs: LuaUserDataRef<Vector3>| {
            Ok(Vector3(this.0.cross(rhs.0)))
        });
        methods.add_method("Dot", |_, this, rhs: LuaUserDataRef<Vector3>| {
            Ok(this.0.dot(rhs.0))
        });
        methods.add_method(
            "FuzzyEq",
            |_, this, (rhs, epsilon): (LuaUserDataRef<Vector3>, f32)| {
                let eq_x = (rhs.0.x - this.0.x).abs() <= epsilon;
                let eq_y = (rhs.0.y - this.0.y).abs() <= epsilon;
                let eq_z = (rhs.0.z - this.0.z).abs() <= epsilon;
                Ok(eq_x && eq_y && eq_z)
            },
        );
        methods.add_method(
            "Lerp",
            |_, this, (rhs, alpha): (LuaUserDataRef<Vector3>, f32)| {
                Ok(Vector3(this.0.lerp(rhs.0, alpha)))
            },
        );
        methods.add_method("Max", |_, this, rhs: LuaUserDataRef<Vector3>| {
            Ok(Vector3(this.0.max(rhs.0)))
        });
        methods.add_method("Min", |_, this, rhs: LuaUserDataRef<Vector3>| {
            Ok(Vector3(this.0.min(rhs.0)))
        });
        methods.add_method("Abs", |_, this, ()| Ok(Vector3(this.0.abs())));
        methods.add_method("Ceil", |_, this, ()| Ok(Vector3(this.0.ceil())));
        methods.add_method("Floor", |_, this, ()| Ok(Vector3(this.0.floor())));
        methods.add_method("Sign", |_, this, ()| Ok(Vector3(this.0.signum())));
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

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.0.x, self.0.y, self.0.z)
    }
}

impl ops::Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Vector3(-self.0)
    }
}

impl ops::Add for Vector3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Vector3(self.0 + rhs.0)
    }
}

impl ops::Sub for Vector3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Vector3(self.0 - rhs.0)
    }
}

impl ops::Mul for Vector3 {
    type Output = Vector3;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl ops::Mul<f32> for Vector3 {
    type Output = Vector3;
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Div for Vector3 {
    type Output = Vector3;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl ops::Div<f32> for Vector3 {
    type Output = Vector3;
    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl IDiv for Vector3 {
    type Output = Vector3;
    fn idiv(self, rhs: Self) -> Self::Output {
        Self((self.0 / rhs.0).floor())
    }
}

impl IDiv<f32> for Vector3 {
    type Output = Vector3;
    fn idiv(self, rhs: f32) -> Self::Output {
        Self((self.0 / rhs).floor())
    }
}

impl From<DomVector3> for Vector3 {
    fn from(v: DomVector3) -> Self {
        Vector3(Vec3 {
            x: v.x,
            y: v.y,
            z: v.z,
        })
    }
}

impl From<Vector3> for DomVector3 {
    fn from(v: Vector3) -> Self {
        DomVector3 {
            x: round_float_decimal(v.0.x),
            y: round_float_decimal(v.0.y),
            z: round_float_decimal(v.0.z),
        }
    }
}
