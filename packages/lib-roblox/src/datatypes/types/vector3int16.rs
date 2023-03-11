use core::fmt;
use std::ops;

use glam::IVec3;
use mlua::prelude::*;
use rbx_dom_weak::types::Vector3int16 as RbxVector3int16;

use super::super::*;

/**
    An implementation of the [Vector3int16](https://create.roblox.com/docs/reference/engine/datatypes/Vector3int16)
    Roblox datatype, backed by [`glam::IVec3`].

    This implements all documented properties, methods &
    constructors of the Vector3int16 class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3int16(pub IVec3);

impl Vector3int16 {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        datatype_table.set(
            "new",
            lua.create_function(|_, (x, y, z): (Option<i16>, Option<i16>, Option<i16>)| {
                Ok(Vector3int16(IVec3 {
                    x: x.unwrap_or_default() as i32,
                    y: y.unwrap_or_default() as i32,
                    z: z.unwrap_or_default() as i32,
                }))
            })?,
        )
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
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, rhs: LuaValue| {
            match &rhs {
                LuaValue::Number(n) => return Ok(Vector3int16(this.0 * IVec3::splat(*n as i32))),
                LuaValue::Integer(i) => return Ok(Vector3int16(this.0 * IVec3::splat(*i))),
                LuaValue::UserData(ud) => {
                    if let Ok(vec) = ud.borrow::<Vector3int16>() {
                        return Ok(Vector3int16(this.0 * vec.0));
                    }
                }
                _ => {}
            };
            Err(LuaError::FromLuaConversionError {
                from: rhs.type_name(),
                to: "Vector3int16",
                message: Some(format!(
                    "Expected Vector3int16 or number, got {}",
                    rhs.type_name()
                )),
            })
        });
        methods.add_meta_method(LuaMetaMethod::Div, |_, this, rhs: LuaValue| {
            match &rhs {
                LuaValue::Number(n) => return Ok(Vector3int16(this.0 / IVec3::splat(*n as i32))),
                LuaValue::Integer(i) => return Ok(Vector3int16(this.0 / IVec3::splat(*i))),
                LuaValue::UserData(ud) => {
                    if let Ok(vec) = ud.borrow::<Vector3int16>() {
                        return Ok(Vector3int16(this.0 / vec.0));
                    }
                }
                _ => {}
            };
            Err(LuaError::FromLuaConversionError {
                from: rhs.type_name(),
                to: "Vector3int16",
                message: Some(format!(
                    "Expected Vector3int16 or number, got {}",
                    rhs.type_name()
                )),
            })
        });
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

impl From<&RbxVector3int16> for Vector3int16 {
    fn from(v: &RbxVector3int16) -> Self {
        Vector3int16(IVec3 {
            x: v.x.clamp(i16::MIN, i16::MAX) as i32,
            y: v.y.clamp(i16::MIN, i16::MAX) as i32,
            z: v.z.clamp(i16::MIN, i16::MAX) as i32,
        })
    }
}

impl From<&Vector3int16> for RbxVector3int16 {
    fn from(v: &Vector3int16) -> Self {
        RbxVector3int16 {
            x: v.0.x.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            y: v.0.y.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            z: v.0.z.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        }
    }
}

impl FromRbxVariant for Vector3int16 {
    fn from_rbx_variant(variant: &RbxVariant) -> DatatypeConversionResult<Self> {
        if let RbxVariant::Vector3int16(v) = variant {
            Ok(v.into())
        } else {
            Err(DatatypeConversionError::FromRbxVariant {
                from: variant.variant_name(),
                to: "Vector3int16",
                detail: None,
            })
        }
    }
}

impl ToRbxVariant for Vector3int16 {
    fn to_rbx_variant(
        &self,
        desired_type: Option<RbxVariantType>,
    ) -> DatatypeConversionResult<RbxVariant> {
        if matches!(desired_type, None | Some(RbxVariantType::Vector3int16)) {
            Ok(RbxVariant::Vector3int16(self.into()))
        } else {
            Err(DatatypeConversionError::ToRbxVariant {
                to: desired_type.map(|d| d.variant_name()).unwrap_or("?"),
                from: "Vector2",
                detail: None,
            })
        }
    }
}
