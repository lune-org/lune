use std::any::type_name;

use mlua::prelude::*;

use rbx_dom_weak::types::{Variant as RbxVariant, VariantType as RbxVariantType};

use crate::datatypes::extension::RbxVariantExt;

use super::*;

pub(crate) trait LuaToRbxVariant<'lua> {
    fn lua_to_rbx_variant(
        &self,
        lua: &'lua Lua,
        variant_type: RbxVariantType,
    ) -> DatatypeConversionResult<RbxVariant>;
}

pub(crate) trait RbxVariantToLua<'lua>: Sized {
    fn rbx_variant_to_lua(lua: &'lua Lua, variant: &RbxVariant) -> DatatypeConversionResult<Self>;
}

/*

    Blanket trait implementations for converting between LuaValue and rbx_dom Variant values

    These should be considered stable and done, already containing all of the known primitives

    See bottom of module for implementations between our custom datatypes and lua userdata

*/

impl<'lua> RbxVariantToLua<'lua> for LuaValue<'lua> {
    fn rbx_variant_to_lua(lua: &'lua Lua, variant: &RbxVariant) -> DatatypeConversionResult<Self> {
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::engine::Engine as _;
        use RbxVariant as Rbx;

        match LuaAnyUserData::rbx_variant_to_lua(lua, variant) {
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

impl<'lua> LuaToRbxVariant<'lua> for LuaValue<'lua> {
    fn lua_to_rbx_variant(
        &self,
        lua: &'lua Lua,
        variant_type: RbxVariantType,
    ) -> DatatypeConversionResult<RbxVariant> {
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::engine::Engine as _;
        use RbxVariantType as Rbx;

        match (self, variant_type) {
            (LuaValue::Boolean(b), Rbx::Bool) => Ok(RbxVariant::Bool(*b)),

            (LuaValue::Integer(i), Rbx::Int64) => Ok(RbxVariant::Int64(*i as i64)),
            (LuaValue::Integer(i), Rbx::Int32) => Ok(RbxVariant::Int32(*i)),
            (LuaValue::Integer(i), Rbx::Float64) => Ok(RbxVariant::Float64(*i as f64)),
            (LuaValue::Integer(i), Rbx::Float32) => Ok(RbxVariant::Float32(*i as f32)),

            (LuaValue::Number(n), Rbx::Int64) => Ok(RbxVariant::Int64(*n as i64)),
            (LuaValue::Number(n), Rbx::Int32) => Ok(RbxVariant::Int32(*n as i32)),
            (LuaValue::Number(n), Rbx::Float64) => Ok(RbxVariant::Float64(*n)),
            (LuaValue::Number(n), Rbx::Float32) => Ok(RbxVariant::Float32(*n as f32)),

            (LuaValue::String(s), Rbx::String) => Ok(RbxVariant::String(s.to_str()?.to_string())),
            (LuaValue::String(s), Rbx::Content) => {
                Ok(RbxVariant::Content(s.to_str()?.to_string().into()))
            }
            (LuaValue::String(s), Rbx::BinaryString) => {
                Ok(RbxVariant::BinaryString(STANDARD_NO_PAD.decode(s)?.into()))
            }

            (LuaValue::UserData(u), d) => u.lua_to_rbx_variant(lua, d),

            (v, d) => Err(DatatypeConversionError::ToRbxVariant {
                to: d.variant_name(),
                from: v.type_name(),
                detail: None,
            }),
        }
    }
}

/*

    Trait implementations for converting between all of
    our custom datatypes and generic Lua userdata values

    NOTE: When adding a new datatype, make sure to add it below to _both_
    of the traits and not just one to allow for bidirectional conversion

*/

impl<'lua> RbxVariantToLua<'lua> for LuaAnyUserData<'lua> {
    #[rustfmt::skip]
    fn rbx_variant_to_lua(lua: &'lua Lua, variant: &RbxVariant) -> DatatypeConversionResult<Self> {
        use super::types::*;
        use RbxVariant as Rbx;

        Ok(match variant.clone() {
            // Not yet implemented datatypes
            // Rbx::Axes(_) => todo!(),
            // Rbx::CFrame(_) => todo!(),
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

            Rbx::BrickColor(value) => lua.create_userdata(BrickColor::from(value))?,

            Rbx::Color3(value)        => lua.create_userdata(Color3::from(value))?,
            Rbx::Color3uint8(value)   => lua.create_userdata(Color3::from(value))?,
            Rbx::ColorSequence(value) => lua.create_userdata(ColorSequence::from(value))?,

            Rbx::UDim(value)  => lua.create_userdata(UDim::from(value))?,
            Rbx::UDim2(value) => lua.create_userdata(UDim2::from(value))?,

            Rbx::Vector2(value)      => lua.create_userdata(Vector2::from(value))?,
            Rbx::Vector2int16(value) => lua.create_userdata(Vector2int16::from(value))?,
            Rbx::Vector3(value)      => lua.create_userdata(Vector3::from(value))?,
            Rbx::Vector3int16(value) => lua.create_userdata(Vector3int16::from(value))?,

            v => {
                return Err(DatatypeConversionError::FromRbxVariant {
                    from: v.variant_name(),
                    to: "userdata",
                    detail: Some("Type not supported".to_string()),
                })
            }
        })
    }
}

impl<'lua> LuaToRbxVariant<'lua> for LuaAnyUserData<'lua> {
    #[rustfmt::skip]
    fn lua_to_rbx_variant(
        &self,
        _: &'lua Lua,
        variant_type: RbxVariantType,
    ) -> DatatypeConversionResult<RbxVariant> {
        use super::types::*;
        use rbx_dom_weak::types as rbx;

        let f = match variant_type {
            RbxVariantType::BrickColor => convert::<BrickColor, rbx::BrickColor>,

            RbxVariantType::Color3        => convert::<Color3,        rbx::Color3>,
            RbxVariantType::Color3uint8   => convert::<Color3,        rbx::Color3uint8>,
            RbxVariantType::ColorSequence => convert::<ColorSequence, rbx::ColorSequence>,

            RbxVariantType::UDim  => convert::<UDim,  rbx::UDim>,
            RbxVariantType::UDim2 => convert::<UDim2, rbx::UDim2>,

            RbxVariantType::Vector2      => convert::<Vector2,      rbx::Vector2>,
            RbxVariantType::Vector2int16 => convert::<Vector2int16, rbx::Vector2int16>,
            RbxVariantType::Vector3      => convert::<Vector3,      rbx::Vector3>,
            RbxVariantType::Vector3int16 => convert::<Vector3int16, rbx::Vector3int16>,

            _ => return Err(DatatypeConversionError::ToRbxVariant {
                to: variant_type.variant_name(),
                from: "userdata",
                detail: Some("Type not supported".to_string()),
            }),
        };

		f(self, variant_type)
    }
}

fn convert<Datatype, RbxType>(
    userdata: &LuaAnyUserData,
    variant_type: RbxVariantType,
) -> DatatypeConversionResult<RbxVariant>
where
    Datatype: LuaUserData + Clone + 'static,
    RbxType: From<Datatype> + Into<RbxVariant>,
{
    match userdata.borrow::<Datatype>() {
        Ok(value) => Ok(RbxType::from(value.clone()).into()),
        Err(LuaError::UserDataTypeMismatch) => Err(DatatypeConversionError::ToRbxVariant {
            to: variant_type.variant_name(),
            from: type_name::<Datatype>(),
            detail: Some("Type mismatch".to_string()),
        }),
        Err(e) => Err(DatatypeConversionError::ToRbxVariant {
            to: variant_type.variant_name(),
            from: type_name::<Datatype>(),
            detail: Some(format!("Internal error: {e}")),
        }),
    }
}
