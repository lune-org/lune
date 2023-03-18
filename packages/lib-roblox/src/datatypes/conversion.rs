use mlua::prelude::*;

use rbx_dom_weak::types::{Variant as DomValue, VariantType as DomType};

use crate::datatypes::extension::DomValueExt;

use super::*;

pub(crate) trait LuaToDomValue<'lua> {
    fn lua_to_dom_value(
        &self,
        lua: &'lua Lua,
        variant_type: DomType,
    ) -> DomConversionResult<DomValue>;
}

pub(crate) trait DomValueToLua<'lua>: Sized {
    fn dom_value_to_lua(lua: &'lua Lua, variant: &DomValue) -> DomConversionResult<Self>;
}

/*

    Blanket trait implementations for converting between LuaValue and rbx_dom Variant values

    These should be considered stable and done, already containing all of the known primitives

    See bottom of module for implementations between our custom datatypes and lua userdata

*/

impl<'lua> DomValueToLua<'lua> for LuaValue<'lua> {
    fn dom_value_to_lua(lua: &'lua Lua, variant: &DomValue) -> DomConversionResult<Self> {
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::engine::Engine as _;

        use rbx_dom_weak::types as dom;

        match LuaAnyUserData::dom_value_to_lua(lua, variant) {
            Ok(value) => Ok(LuaValue::UserData(value)),
            Err(e) => match variant {
                DomValue::Bool(b) => Ok(LuaValue::Boolean(*b)),
                DomValue::Int64(i) => Ok(LuaValue::Number(*i as f64)),
                DomValue::Int32(i) => Ok(LuaValue::Number(*i as f64)),
                DomValue::Float64(n) => Ok(LuaValue::Number(*n)),
                DomValue::Float32(n) => Ok(LuaValue::Number(*n as f64)),
                DomValue::String(s) => Ok(LuaValue::String(lua.create_string(s)?)),
                DomValue::Content(s) => Ok(LuaValue::String(
                    lua.create_string(AsRef::<str>::as_ref(s))?,
                )),
                DomValue::BinaryString(s) => {
                    let encoded = STANDARD_NO_PAD.encode(AsRef::<[u8]>::as_ref(s));
                    Ok(LuaValue::String(lua.create_string(&encoded)?))
                }

                // NOTE: We need this special case here to handle default (nil)
                // physical properties since our PhysicalProperties datatype
                // implementation does not handle default at all, only custom
                DomValue::PhysicalProperties(dom::PhysicalProperties::Default) => Ok(LuaValue::Nil),

                _ => Err(e),
            },
        }
    }
}

impl<'lua> LuaToDomValue<'lua> for LuaValue<'lua> {
    fn lua_to_dom_value(
        &self,
        lua: &'lua Lua,
        variant_type: DomType,
    ) -> DomConversionResult<DomValue> {
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::engine::Engine as _;

        use rbx_dom_weak::types as dom;

        match (self, variant_type) {
            (LuaValue::Boolean(b), DomType::Bool) => Ok(DomValue::Bool(*b)),

            (LuaValue::Integer(i), DomType::Int64) => Ok(DomValue::Int64(*i as i64)),
            (LuaValue::Integer(i), DomType::Int32) => Ok(DomValue::Int32(*i)),
            (LuaValue::Integer(i), DomType::Float64) => Ok(DomValue::Float64(*i as f64)),
            (LuaValue::Integer(i), DomType::Float32) => Ok(DomValue::Float32(*i as f32)),

            (LuaValue::Number(n), DomType::Int64) => Ok(DomValue::Int64(*n as i64)),
            (LuaValue::Number(n), DomType::Int32) => Ok(DomValue::Int32(*n as i32)),
            (LuaValue::Number(n), DomType::Float64) => Ok(DomValue::Float64(*n)),
            (LuaValue::Number(n), DomType::Float32) => Ok(DomValue::Float32(*n as f32)),

            (LuaValue::String(s), DomType::String) => Ok(DomValue::String(s.to_str()?.to_string())),
            (LuaValue::String(s), DomType::Content) => {
                Ok(DomValue::Content(s.to_str()?.to_string().into()))
            }
            (LuaValue::String(s), DomType::BinaryString) => {
                Ok(DomValue::BinaryString(STANDARD_NO_PAD.decode(s)?.into()))
            }

            // NOTE: We need this special case here to handle default (nil)
            // physical properties since our PhysicalProperties datatype
            // implementation does not handle default at all, only custom
            (LuaValue::Nil, DomType::PhysicalProperties) => Ok(DomValue::PhysicalProperties(
                dom::PhysicalProperties::Default,
            )),

            (LuaValue::UserData(u), d) => u.lua_to_dom_value(lua, d),

            (v, d) => Err(DomConversionError::ToDomValue {
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

impl<'lua> DomValueToLua<'lua> for LuaAnyUserData<'lua> {
    #[rustfmt::skip]
    fn dom_value_to_lua(lua: &'lua Lua, variant: &DomValue) -> DomConversionResult<Self> {
		use super::types::*;

        use rbx_dom_weak::types as dom;

        /*
            NOTES:

            1. Enum is intentionally left out here, it has a custom
               conversion going from instance property > datatype instead,
               check `EnumItem::from_instance_property` for specifics

            2. PhysicalProperties can only be converted if they are custom
               physical properties, since default physical properties values
               depend on what other related properties an instance might have

        */
        Ok(match variant.clone() {
            DomValue::Axes(value)  => lua.create_userdata(Axes::from(value))?,
            DomValue::Faces(value) => lua.create_userdata(Faces::from(value))?,

            DomValue::CFrame(value) => lua.create_userdata(CFrame::from(value))?,

            DomValue::BrickColor(value)    => lua.create_userdata(BrickColor::from(value))?,
            DomValue::Color3(value)        => lua.create_userdata(Color3::from(value))?,
            DomValue::Color3uint8(value)   => lua.create_userdata(Color3::from(value))?,
            DomValue::ColorSequence(value) => lua.create_userdata(ColorSequence::from(value))?,

            DomValue::Font(value) => lua.create_userdata(Font::from(value))?,

            DomValue::NumberRange(value)    => lua.create_userdata(NumberRange::from(value))?,
            DomValue::NumberSequence(value) => lua.create_userdata(NumberSequence::from(value))?,

            DomValue::Ray(value) => lua.create_userdata(Ray::from(value))?,

            DomValue::Rect(value)  => lua.create_userdata(Rect::from(value))?,
            DomValue::UDim(value)  => lua.create_userdata(UDim::from(value))?,
            DomValue::UDim2(value) => lua.create_userdata(UDim2::from(value))?,

            DomValue::Region3(value)      => lua.create_userdata(Region3::from(value))?,
            DomValue::Region3int16(value) => lua.create_userdata(Region3int16::from(value))?,
            DomValue::Vector2(value)      => lua.create_userdata(Vector2::from(value))?,
            DomValue::Vector2int16(value) => lua.create_userdata(Vector2int16::from(value))?,
            DomValue::Vector3(value)      => lua.create_userdata(Vector3::from(value))?,
            DomValue::Vector3int16(value) => lua.create_userdata(Vector3int16::from(value))?,

            DomValue::OptionalCFrame(value) => match value {
                Some(value) => lua.create_userdata(CFrame::from(value))?,
                None => lua.create_userdata(CFrame::IDENTITY)?
            },

            DomValue::PhysicalProperties(dom::PhysicalProperties::Custom(value)) => {
                lua.create_userdata(PhysicalProperties::from(value))?
            },

            v => {
                return Err(DomConversionError::FromDomValue {
                    from: v.variant_name(),
                    to: "userdata",
                    detail: Some("Type not supported".to_string()),
                })
            }
        })
    }
}

impl<'lua> LuaToDomValue<'lua> for LuaAnyUserData<'lua> {
    #[rustfmt::skip]
    fn lua_to_dom_value(
        &self,
        _: &'lua Lua,
        variant_type: DomType,
    ) -> DomConversionResult<DomValue> {
        use super::types::*;

        use rbx_dom_weak::types as dom;

        let f = match variant_type {
            DomType::Axes  => convert::<Axes,  dom::Axes>,
            DomType::Faces => convert::<Faces, dom::Faces>,

            DomType::CFrame => convert::<CFrame, dom::CFrame>,

            DomType::BrickColor    => convert::<BrickColor,    dom::BrickColor>,
            DomType::Color3        => convert::<Color3,        dom::Color3>,
            DomType::Color3uint8   => convert::<Color3,        dom::Color3uint8>,
            DomType::ColorSequence => convert::<ColorSequence, dom::ColorSequence>,

            DomType::Enum => convert::<EnumItem, dom::Enum>,

            DomType::Font => convert::<Font, dom::Font>,

            DomType::NumberRange    => convert::<NumberRange,    dom::NumberRange>,
            DomType::NumberSequence => convert::<NumberSequence, dom::NumberSequence>,

            DomType::Rect  => convert::<Rect,  dom::Rect>,
            DomType::UDim  => convert::<UDim,  dom::UDim>,
            DomType::UDim2 => convert::<UDim2, dom::UDim2>,

            DomType::Ray => convert::<Ray, dom::Ray>,

            DomType::Region3      => convert::<Region3,      dom::Region3>,
            DomType::Region3int16 => convert::<Region3int16, dom::Region3int16>,
            DomType::Vector2      => convert::<Vector2,      dom::Vector2>,
            DomType::Vector2int16 => convert::<Vector2int16, dom::Vector2int16>,
            DomType::Vector3      => convert::<Vector3,      dom::Vector3>,
            DomType::Vector3int16 => convert::<Vector3int16, dom::Vector3int16>,

            DomType::OptionalCFrame => return match self.borrow::<CFrame>() {
                Ok(value) => Ok(DomValue::OptionalCFrame(Some(dom::CFrame::from(*value)))),
                Err(e) => Err(lua_userdata_error_to_conversion_error(variant_type, e)),
            },

            DomType::PhysicalProperties => return match self.borrow::<PhysicalProperties>() {
                Ok(value) => {
                    let props = dom::CustomPhysicalProperties::from(*value);
                    let custom = dom::PhysicalProperties::Custom(props);
                    Ok(DomValue::PhysicalProperties(custom))
                },
                Err(e) => Err(lua_userdata_error_to_conversion_error(variant_type, e)),
            },

            _ => return Err(DomConversionError::ToDomValue {
                to: variant_type.variant_name(),
                from: "userdata",
                detail: Some("Type not supported".to_string()),
            }),
        };

        f(self, variant_type)
    }
}

fn convert<TypeFrom, TypeTo>(
    userdata: &LuaAnyUserData,
    variant_type: DomType,
) -> DomConversionResult<DomValue>
where
    TypeFrom: LuaUserData + Clone + 'static,
    TypeTo: From<TypeFrom> + Into<DomValue>,
{
    match userdata.borrow::<TypeFrom>() {
        Ok(value) => Ok(TypeTo::from(value.clone()).into()),
        Err(e) => Err(lua_userdata_error_to_conversion_error(variant_type, e)),
    }
}

fn lua_userdata_error_to_conversion_error(
    variant_type: DomType,
    error: LuaError,
) -> DomConversionError {
    match error {
        LuaError::UserDataTypeMismatch => DomConversionError::ToDomValue {
            to: variant_type.variant_name(),
            from: "userdata",
            detail: Some("Type mismatch".to_string()),
        },
        e => DomConversionError::ToDomValue {
            to: variant_type.variant_name(),
            from: "userdata",
            detail: Some(format!("Internal error: {e}")),
        },
    }
}
