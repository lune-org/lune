use core::fmt;

use glam::IVec2;
use mlua::prelude::*;
use rbx_dom_weak::types::Vector2int16 as RbxVector2int16;

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

impl fmt::Display for Vector2int16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.0.x, self.0.y)
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
        methods.add_meta_method(LuaMetaMethod::Unm, |_, this, ()| Ok(Vector2int16(-this.0)));
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, rhs: Vector2int16| {
            Ok(Vector2int16(this.0 + rhs.0))
        });
        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, rhs: Vector2int16| {
            Ok(Vector2int16(this.0 - rhs.0))
        });
        methods.add_meta_method(LuaMetaMethod::Mul, |_, this, rhs: LuaValue| {
            match &rhs {
                LuaValue::Number(n) => return Ok(Vector2int16(this.0 * IVec2::splat(*n as i32))),
                LuaValue::Integer(i) => return Ok(Vector2int16(this.0 * IVec2::splat(*i))),
                LuaValue::UserData(ud) => {
                    if let Ok(vec) = ud.borrow::<Vector2int16>() {
                        return Ok(Vector2int16(this.0 * vec.0));
                    }
                }
                _ => {}
            };
            Err(LuaError::FromLuaConversionError {
                from: rhs.type_name(),
                to: "Vector2int16",
                message: Some(format!(
                    "Expected Vector2int16 or number, got {}",
                    rhs.type_name()
                )),
            })
        });
        methods.add_meta_method(LuaMetaMethod::Div, |_, this, rhs: LuaValue| {
            match &rhs {
                LuaValue::Number(n) => return Ok(Vector2int16(this.0 / IVec2::splat(*n as i32))),
                LuaValue::Integer(i) => return Ok(Vector2int16(this.0 / IVec2::splat(*i))),
                LuaValue::UserData(ud) => {
                    if let Ok(vec) = ud.borrow::<Vector2int16>() {
                        return Ok(Vector2int16(this.0 / vec.0));
                    }
                }
                _ => {}
            };
            Err(LuaError::FromLuaConversionError {
                from: rhs.type_name(),
                to: "Vector2int16",
                message: Some(format!(
                    "Expected Vector2int16 or number, got {}",
                    rhs.type_name()
                )),
            })
        });
    }
}

impl From<&RbxVector2int16> for Vector2int16 {
    fn from(v: &RbxVector2int16) -> Self {
        Vector2int16(IVec2 {
            x: v.x.clamp(i16::MIN, i16::MAX) as i32,
            y: v.y.clamp(i16::MIN, i16::MAX) as i32,
        })
    }
}

impl From<&Vector2int16> for RbxVector2int16 {
    fn from(v: &Vector2int16) -> Self {
        RbxVector2int16 {
            x: v.0.x.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
            y: v.0.y.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        }
    }
}

impl FromRbxVariant for Vector2int16 {
    fn from_rbx_variant(variant: &RbxVariant) -> DatatypeConversionResult<Self> {
        if let RbxVariant::Vector2int16(v) = variant {
            Ok(v.into())
        } else {
            Err(DatatypeConversionError::FromRbxVariant {
                from: variant.variant_name(),
                to: "Vector2int16",
                detail: None,
            })
        }
    }
}

impl ToRbxVariant for Vector2int16 {
    fn to_rbx_variant(
        &self,
        desired_type: Option<RbxVariantType>,
    ) -> DatatypeConversionResult<RbxVariant> {
        if matches!(desired_type, None | Some(RbxVariantType::Vector2int16)) {
            Ok(RbxVariant::Vector2int16(self.into()))
        } else {
            Err(DatatypeConversionError::ToRbxVariant {
                to: desired_type.map(|d| d.variant_name()).unwrap_or("?"),
                from: "Vector2",
                detail: None,
            })
        }
    }
}
