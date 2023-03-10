use core::fmt;

use glam::Vec3A;
use mlua::prelude::*;
use rbx_dom_weak::types::Vector3 as RbxVector3;

use super::*;

/**
    An implementation of the [Vector3](https://create.roblox.com/docs/reference/engine/datatypes/Vector3)
    Roblox datatype, backed by [`glam::Vec3A`].

    This implements all documented properties & methods of the Vector3
    class as of March 2023, as well as the `new(x, y, z)` constructor.

    Note that this does not use native Luau vectors to simplify implementation
    and instead allow us to implement all abovementioned APIs accurately.
*/
#[derive(Debug, Clone, Copy)]
pub struct Vector3(pub Vec3A);

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.0.x, self.0.y, self.0.z)
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
        // Metamethods - normal
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(this.to_string()));
        methods.add_meta_method(LuaMetaMethod::Eq, |_, this, rhs: LuaValue| {
            if let LuaValue::UserData(ud) = rhs {
                if let Ok(vec) = ud.borrow::<Vector3>() {
                    Ok(this.0 == vec.0)
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        });
        // Metamethods - math
        methods.add_meta_method(LuaMetaMethod::Unm, |_, this, ()| Ok(Vector3(-this.0)));
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, rhs: Vector3| {
            Ok(Vector3(this.0 + rhs.0))
        });
        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, rhs: Vector3| {
            Ok(Vector3(this.0 - rhs.0))
        });
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, rhs: LuaValue| {
            match &rhs {
                LuaValue::Number(n) => return Ok(Vector3(this.0 * Vec3A::splat(*n as f32))),
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
                LuaValue::Number(n) => return Ok(Vector3(this.0 / Vec3A::splat(*n as f32))),
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

impl DatatypeTable for Vector3 {
    fn make_dt_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        // Constants
        datatype_table.set("xAxis", Vector3(Vec3A::X))?;
        datatype_table.set("yAxis", Vector3(Vec3A::Y))?;
        datatype_table.set("zAxis", Vector3(Vec3A::Z))?;
        datatype_table.set("zero", Vector3(Vec3A::ZERO))?;
        datatype_table.set("one", Vector3(Vec3A::ONE))?;
        // Constructors
        datatype_table.set(
            "new",
            lua.create_function(|_, (x, y, z): (Option<f32>, Option<f32>, Option<f32>)| {
                Ok(Vector3(Vec3A {
                    x: x.unwrap_or_default(),
                    y: y.unwrap_or_default(),
                    z: z.unwrap_or_default(),
                }))
            })?,
        )
        // FUTURE: Implement FromNormalId and FromAxis constructors?
    }
}

impl FromRbxVariant for Vector3 {
    fn from_rbx_variant(variant: &RbxVariant) -> RbxConversionResult<Self> {
        if let RbxVariant::Vector3(v) = variant {
            Ok(Vector3(Vec3A {
                x: v.x,
                y: v.y,
                z: v.z,
            }))
        } else {
            Err(RbxConversionError::FromRbxVariant {
                from: variant.display_name(),
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
    ) -> RbxConversionResult<RbxVariant> {
        if matches!(desired_type, None | Some(RbxVariantType::Vector3)) {
            Ok(RbxVariant::Vector3(RbxVector3 {
                x: self.0.x,
                y: self.0.y,
                z: self.0.z,
            }))
        } else {
            Err(RbxConversionError::DesiredTypeMismatch {
                actual: RbxVariantType::Vector3.display_name(),
                detail: None,
            })
        }
    }
}
