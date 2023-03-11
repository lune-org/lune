use mlua::prelude::*;

use rbx_dom_weak::types::{Variant as RbxVariant, VariantType as RbxVariantType};

use crate::datatypes::extension::RbxVariantExt;

use super::*;

pub(crate) trait ToRbxVariant {
    fn to_rbx_variant(
        &self,
        desired_type: Option<RbxVariantType>,
    ) -> DatatypeConversionResult<RbxVariant>;
}

pub(crate) trait FromRbxVariant: Sized {
    fn from_rbx_variant(variant: &RbxVariant) -> DatatypeConversionResult<Self>;
}

pub(crate) trait FromRbxVariantLua<'lua>: Sized {
    fn from_rbx_variant_lua(variant: &RbxVariant, lua: &'lua Lua)
        -> DatatypeConversionResult<Self>;
}

/*

    Blanket trait implementations for converting between LuaValue and rbx_dom Variant values

    These should be considered stable and one, already containing all of the primitive types

    See bottom of module for implementations between our custom datatypes and lua userdata

*/

impl<'lua> FromRbxVariantLua<'lua> for LuaValue<'lua> {
    fn from_rbx_variant_lua(
        variant: &RbxVariant,
        lua: &'lua Lua,
    ) -> DatatypeConversionResult<Self> {
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::engine::Engine as _;
        use RbxVariant as Rbx;

        match LuaAnyUserData::from_rbx_variant_lua(variant, lua) {
            Ok(value) => Ok(LuaValue::UserData(value)),
            Err(e) => match variant {
                Rbx::Bool(b) => Ok(LuaValue::Boolean(*b)),
                Rbx::Int64(i) => Ok(LuaValue::Number(*i as f64)),
                Rbx::Int32(i) => Ok(LuaValue::Number(*i as f64)),
                Rbx::Float64(n) => Ok(LuaValue::Number(*n)),
                Rbx::Float32(n) => Ok(LuaValue::Number(*n as f64)),
                Rbx::String(s) => Ok(LuaValue::String(lua.create_string(s)?)),
                Rbx::Content(s) => Ok(LuaValue::String(
                    lua.create_string(AsRef::<str>::as_ref(s))?,
                )),
                Rbx::BinaryString(s) => {
                    let encoded = STANDARD_NO_PAD.encode(AsRef::<[u8]>::as_ref(s));
                    Ok(LuaValue::String(lua.create_string(&encoded)?))
                }
                _ => Err(e),
            },
        }
    }
}

impl<'lua> ToRbxVariant for LuaValue<'lua> {
    fn to_rbx_variant(
        &self,
        desired_type: Option<RbxVariantType>,
    ) -> DatatypeConversionResult<RbxVariant> {
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::engine::Engine as _;
        use RbxVariantType as Rbx;

        if let Some(desired_type) = desired_type {
            match (self, desired_type) {
                (LuaValue::Boolean(b), Rbx::Bool) => Ok(RbxVariant::Bool(*b)),
                (LuaValue::Integer(i), Rbx::Int64) => Ok(RbxVariant::Int64(*i as i64)),
                (LuaValue::Integer(i), Rbx::Int32) => Ok(RbxVariant::Int32(*i)),
                (LuaValue::Integer(i), Rbx::Float64) => Ok(RbxVariant::Float64(*i as f64)),
                (LuaValue::Integer(i), Rbx::Float32) => Ok(RbxVariant::Float32(*i as f32)),
                (LuaValue::Number(n), Rbx::Int64) => Ok(RbxVariant::Int64(*n as i64)),
                (LuaValue::Number(n), Rbx::Int32) => Ok(RbxVariant::Int32(*n as i32)),
                (LuaValue::Number(n), Rbx::Float64) => Ok(RbxVariant::Float64(*n)),
                (LuaValue::Number(n), Rbx::Float32) => Ok(RbxVariant::Float32(*n as f32)),
                (LuaValue::String(s), Rbx::String) => {
                    Ok(RbxVariant::String(s.to_str()?.to_string()))
                }
                (LuaValue::String(s), Rbx::Content) => {
                    Ok(RbxVariant::Content(s.to_str()?.to_string().into()))
                }
                (LuaValue::String(s), Rbx::BinaryString) => {
                    Ok(RbxVariant::BinaryString(STANDARD_NO_PAD.decode(s)?.into()))
                }
                (LuaValue::UserData(u), d) => u.to_rbx_variant(Some(d)),
                (v, d) => Err(DatatypeConversionError::ToRbxVariant {
                    to: d.variant_name(),
                    from: v.type_name(),
                    detail: None,
                }),
            }
        } else {
            match self {
                // Primitives
                LuaValue::Boolean(b) => Ok(RbxVariant::Bool(*b)),
                LuaValue::Integer(i) => Ok(RbxVariant::Int32(*i)),
                LuaValue::Number(n) => Ok(RbxVariant::Float64(*n)),
                LuaValue::String(s) => Ok(RbxVariant::String(s.to_str()?.to_string())),
                LuaValue::UserData(u) => u.to_rbx_variant(None),
                v => Err(DatatypeConversionError::ToRbxVariant {
                    to: "Variant",
                    from: v.type_name(),
                    detail: None,
                }),
            }
        }
    }
}

/*

    Trait implementations for converting between all of
    our custom datatypes and generic Lua userdata values

    NOTE: When adding a new datatype, make sure to add it below to _both_
    of the traits and not just one to allow for bidirectional conversion

*/

impl<'lua> FromRbxVariantLua<'lua> for LuaAnyUserData<'lua> {
    #[rustfmt::skip]
    fn from_rbx_variant_lua(variant: &RbxVariant, lua: &'lua Lua) -> DatatypeConversionResult<Self> {
        use RbxVariant as Rbx;
        use super::types::*;
        match variant {
            Rbx::Vector2(_)      => Ok(lua.create_userdata(Vector2::from_rbx_variant(variant)?)?),
            Rbx::Vector2int16(_) => Ok(lua.create_userdata(Vector2int16::from_rbx_variant(variant)?)?),
            Rbx::Vector3(_)      => Ok(lua.create_userdata(Vector3::from_rbx_variant(variant)?)?),
            Rbx::Vector3int16(_) => Ok(lua.create_userdata(Vector3int16::from_rbx_variant(variant)?)?),
            // Not yet implemented datatypes
            // Rbx::Axes(_) => todo!(),
            // Rbx::BrickColor(_) => todo!(),
            // Rbx::CFrame(_) => todo!(),
            // Rbx::Color3(_) => todo!(),
            // Rbx::Color3uint8(_) => todo!(),
            // Rbx::ColorSequence(_) => todo!(),
            // Rbx::Enum(_) => todo!(),
            // Rbx::Faces(_) => todo!(),
            // Rbx::NumberRange(_) => todo!(),
            // Rbx::NumberSequence(_) => todo!(),
            // Rbx::OptionalCFrame(_) => todo!(),
            // Rbx::PhysicalProperties(_) => todo!(),
            // Rbx::Ray(_) => todo!(),
            // Rbx::Rect(_) => todo!(),
            // Rbx::Region3(_) => todo!(),
            // Rbx::Region3int16(_) => todo!(),
            // Rbx::UDim(_) => todo!(),
            // Rbx::UDim2(_) => todo!(),
            v => Err(DatatypeConversionError::FromRbxVariant {
                from: v.variant_name(),
                to: "LuaValue",
                detail: Some("Type not supported".to_string()),
            }),
        }
    }
}

impl<'lua> ToRbxVariant for LuaAnyUserData<'lua> {
    fn to_rbx_variant(
        &self,
        desired_type: Option<RbxVariantType>,
    ) -> DatatypeConversionResult<RbxVariant> {
        use super::types::*;
        if let Ok(v2) = self.borrow::<Vector2>() {
            v2.to_rbx_variant(desired_type)
        } else if let Ok(v2i) = self.borrow::<Vector2int16>() {
            v2i.to_rbx_variant(desired_type)
        } else if let Ok(v3) = self.borrow::<Vector3>() {
            v3.to_rbx_variant(desired_type)
        } else if let Ok(v3i) = self.borrow::<Vector3int16>() {
            v3i.to_rbx_variant(desired_type)
        } else {
            Err(DatatypeConversionError::ToRbxVariant {
                to: desired_type.map(|d| d.variant_name()).unwrap_or("Variant"),
                from: "userdata",
                detail: None,
            })
        }
    }
}
