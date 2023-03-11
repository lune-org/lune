use core::fmt;

use glam::{Vec2, Vec3};
use mlua::prelude::*;
use rbx_dom_weak::types::Vector2 as RbxVector2;

use super::*;

/**
    An implementation of the [Vector2](https://create.roblox.com/docs/reference/engine/datatypes/Vector2)
    Roblox datatype, backed by [`glam::Vec2`].

    This implements all documented properties, methods &
    constructors of the Vector2 class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2(pub Vec2);

impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.0.x, self.0.y)
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
        methods.add_meta_method(LuaMetaMethod::Eq, datatype_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, datatype_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Unm, |_, this, ()| Ok(Vector2(-this.0)));
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, rhs: Vector2| {
            Ok(Vector2(this.0 + rhs.0))
        });
        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, rhs: Vector2| {
            Ok(Vector2(this.0 - rhs.0))
        });
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, rhs: LuaValue| {
            match &rhs {
                LuaValue::Number(n) => return Ok(Vector2(this.0 * Vec2::splat(*n as f32))),
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

impl DatatypeTable for Vector2 {
    fn make_dt_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
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

impl FromRbxVariant for Vector2 {
    fn from_rbx_variant(variant: &RbxVariant) -> RbxConversionResult<Self> {
        if let RbxVariant::Vector2(v) = variant {
            Ok(Vector2(Vec2 { x: v.x, y: v.y }))
        } else {
            Err(RbxConversionError::FromRbxVariant {
                from: variant.display_name(),
                to: "Vector2",
                detail: None,
            })
        }
    }
}

impl ToRbxVariant for Vector2 {
    fn to_rbx_variant(
        &self,
        desired_type: Option<RbxVariantType>,
    ) -> RbxConversionResult<RbxVariant> {
        if matches!(desired_type, None | Some(RbxVariantType::Vector2)) {
            Ok(RbxVariant::Vector2(RbxVector2 {
                x: self.0.x,
                y: self.0.y,
            }))
        } else {
            Err(RbxConversionError::DesiredTypeMismatch {
                can_convert_to: Some(RbxVariantType::Vector2.display_name()),
                detail: None,
            })
        }
    }
}
