use core::fmt;
use std::ops;

use glam::Vec3;
use mlua::prelude::*;
use rbx_dom_weak::types::Vector3 as RbxVector3;

use super::super::*;

/**
    An implementation of the [Vector3](https://create.roblox.com/docs/reference/engine/datatypes/Vector3)
    Roblox datatype, backed by [`glam::Vec3`].

    This implements all documented properties & methods of the Vector3
    class as of March 2023, as well as the `new(x, y, z)` constructor.

    Note that this does not use native Luau vectors to simplify implementation
    and instead allow us to implement all abovementioned APIs accurately.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3(pub Vec3);

impl Vector3 {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        // Constants
        datatype_table.set("xAxis", Vector3(Vec3::X))?;
        datatype_table.set("yAxis", Vector3(Vec3::Y))?;
        datatype_table.set("zAxis", Vector3(Vec3::Z))?;
        datatype_table.set("zero", Vector3(Vec3::ZERO))?;
        datatype_table.set("one", Vector3(Vec3::ONE))?;
        // Constructors
        datatype_table.set(
            "new",
            lua.create_function(|_, (x, y, z): (Option<f32>, Option<f32>, Option<f32>)| {
                Ok(Vector3(Vec3 {
                    x: x.unwrap_or_default(),
                    y: y.unwrap_or_default(),
                    z: z.unwrap_or_default(),
                }))
            })?,
        )
        // FUTURE: Implement FromNormalId and FromAxis constructors?
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
        methods.add_method("Angle", |_, this, rhs: Vector3| {
            Ok(this.0.angle_between(rhs.0))
        });
        methods.add_method("Cross", |_, this, rhs: Vector3| {
            Ok(Vector3(this.0.cross(rhs.0)))
        });
        methods.add_method("Dot", |_, this, rhs: Vector3| Ok(this.0.dot(rhs.0)));
        methods.add_method("FuzzyEq", |_, this, (rhs, epsilon): (Vector3, f32)| {
            let eq_x = (rhs.0.x - this.0.x).abs() <= epsilon;
            let eq_y = (rhs.0.y - this.0.y).abs() <= epsilon;
            let eq_z = (rhs.0.z - this.0.z).abs() <= epsilon;
            Ok(eq_x && eq_y && eq_z)
        });
        methods.add_method("Lerp", |_, this, (rhs, alpha): (Vector3, f32)| {
            Ok(Vector3(this.0.lerp(rhs.0, alpha)))
        });
        methods.add_method("Max", |_, this, rhs: Vector3| {
            Ok(Vector3(this.0.max(rhs.0)))
        });
        methods.add_method("Min", |_, this, rhs: Vector3| {
            Ok(Vector3(this.0.min(rhs.0)))
        });
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Unm, userdata_impl_unm);
        methods.add_meta_method(LuaMetaMethod::Add, userdata_impl_add);
        methods.add_meta_method(LuaMetaMethod::Sub, userdata_impl_sub);
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, rhs: LuaValue| {
            match &rhs {
                LuaValue::Number(n) => return Ok(Vector3(this.0 * Vec3::splat(*n as f32))),
                LuaValue::Integer(i) => return Ok(Vector3(this.0 * Vec3::splat(*i as f32))),
                LuaValue::UserData(ud) => {
                    if let Ok(vec) = ud.borrow::<Vector3>() {
                        return Ok(Vector3(this.0 * vec.0));
                    }
                }
                _ => {}
            };
            Err(LuaError::FromLuaConversionError {
                from: rhs.type_name(),
                to: "Vector3",
                message: Some(format!(
                    "Expected Vector3 or number, got {}",
                    rhs.type_name()
                )),
            })
        });
        methods.add_meta_method(LuaMetaMethod::Div, |_, this, rhs: LuaValue| {
            match &rhs {
                LuaValue::Number(n) => return Ok(Vector3(this.0 / Vec3::splat(*n as f32))),
                LuaValue::Integer(i) => return Ok(Vector3(this.0 / Vec3::splat(*i as f32))),
                LuaValue::UserData(ud) => {
                    if let Ok(vec) = ud.borrow::<Vector3>() {
                        return Ok(Vector3(this.0 / vec.0));
                    }
                }
                _ => {}
            };
            Err(LuaError::FromLuaConversionError {
                from: rhs.type_name(),
                to: "Vector3",
                message: Some(format!(
                    "Expected Vector3 or number, got {}",
                    rhs.type_name()
                )),
            })
        });
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

impl From<&RbxVector3> for Vector3 {
    fn from(v: &RbxVector3) -> Self {
        Vector3(Vec3 {
            x: v.x,
            y: v.y,
            z: v.z,
        })
    }
}

impl From<&Vector3> for RbxVector3 {
    fn from(v: &Vector3) -> Self {
        RbxVector3 {
            x: v.0.x,
            y: v.0.y,
            z: v.0.z,
        }
    }
}

impl FromRbxVariant for Vector3 {
    fn from_rbx_variant(variant: &RbxVariant) -> DatatypeConversionResult<Self> {
        if let RbxVariant::Vector3(v) = variant {
            Ok(v.into())
        } else {
            Err(DatatypeConversionError::FromRbxVariant {
                from: variant.variant_name(),
                to: "Vector3",
                detail: None,
            })
        }
    }
}

impl ToRbxVariant for Vector3 {
    fn to_rbx_variant(
        &self,
        desired_type: Option<RbxVariantType>,
    ) -> DatatypeConversionResult<RbxVariant> {
        if matches!(desired_type, None | Some(RbxVariantType::Vector3)) {
            Ok(RbxVariant::Vector3(self.into()))
        } else {
            Err(DatatypeConversionError::ToRbxVariant {
                to: desired_type.map(|d| d.variant_name()).unwrap_or("?"),
                from: "Vector2",
                detail: None,
            })
        }
    }
}
