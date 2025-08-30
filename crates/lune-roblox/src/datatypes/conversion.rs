use mlua::prelude::*;

use rbx_dom_weak::types::{Variant as DomValue, VariantType as DomType};

use crate::{datatypes::extension::DomValueExt, instance::Instance};

use super::*;

pub(crate) trait LuaToDomValue {
    /**
        Converts a lua value into a weak dom value.

        If a `variant_type` is given the conversion will be more strict
        and also more accurate, it should be given whenever possible.
    */
    fn lua_to_dom_value(
        &self,
        lua: &Lua,
        variant_type: Option<DomType>,
    ) -> DomConversionResult<DomValue>;
}

pub(crate) trait DomValueToLua: Sized {
    /**
        Converts a weak dom value into a lua value.
    */
    fn dom_value_to_lua(lua: &Lua, variant: &DomValue) -> DomConversionResult<Self>;
}

/*

    Blanket trait implementations for converting between LuaValue and rbx_dom Variant values

    These should be considered stable and done, already containing all of the known primitives

    See bottom of module for implementations between our custom datatypes and lua userdata

*/

impl DomValueToLua for LuaValue {
    fn dom_value_to_lua(lua: &Lua, variant: &DomValue) -> DomConversionResult<Self> {
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
                DomValue::BinaryString(s) => Ok(LuaValue::String(lua.create_string(s)?)),
                DomValue::ContentId(s) => Ok(LuaValue::String(
                    lua.create_string(AsRef::<str>::as_ref(s))?,
                )),

                // NOTE: Dom references may point to instances that
                // no longer exist, so we handle that here instead of
                // in the userdata conversion to be able to return nils
                DomValue::Ref(value) => match Instance::new_opt(*value) {
                    Some(inst) => Ok(inst.into_lua(lua)?),
                    None => Ok(LuaValue::Nil),
                },

                // NOTE: Some values are either optional or default and we should handle
                // that properly here since the userdata conversion above will always fail
                DomValue::OptionalCFrame(None)
                | DomValue::PhysicalProperties(dom::PhysicalProperties::Default) => {
                    Ok(LuaValue::Nil)
                }

                _ => Err(e),
            },
        }
    }
}

impl LuaToDomValue for LuaValue {
    fn lua_to_dom_value(
        &self,
        lua: &Lua,
        variant_type: Option<DomType>,
    ) -> DomConversionResult<DomValue> {
        use rbx_dom_weak::types as dom;

        if let Some(variant_type) = variant_type {
            match (self, variant_type) {
                (LuaValue::Boolean(b), DomType::Bool) => Ok(DomValue::Bool(*b)),

                (LuaValue::Integer(i), DomType::Int64) => Ok(DomValue::Int64(*i)),
                (LuaValue::Integer(i), DomType::Int32) => Ok(DomValue::Int32(*i as i32)),
                (LuaValue::Integer(i), DomType::Float64) => Ok(DomValue::Float64(*i as f64)),
                (LuaValue::Integer(i), DomType::Float32) => Ok(DomValue::Float32(*i as f32)),

                (LuaValue::Number(n), DomType::Int64) => Ok(DomValue::Int64(*n as i64)),
                (LuaValue::Number(n), DomType::Int32) => Ok(DomValue::Int32(*n as i32)),
                (LuaValue::Number(n), DomType::Float64) => Ok(DomValue::Float64(*n)),
                (LuaValue::Number(n), DomType::Float32) => Ok(DomValue::Float32(*n as f32)),

                (LuaValue::String(s), DomType::String) => {
                    Ok(DomValue::String(s.to_str()?.to_string()))
                }
                (LuaValue::String(s), DomType::BinaryString) => {
                    Ok(DomValue::BinaryString(s.as_bytes().to_vec().into()))
                }
                (LuaValue::String(s), DomType::ContentId) => {
                    Ok(DomValue::ContentId(s.to_str()?.to_string().into()))
                }

                // NOTE: Some values are either optional or default and we
                // should handle that here before trying to convert as userdata
                (LuaValue::Nil, DomType::OptionalCFrame) => Ok(DomValue::OptionalCFrame(None)),
                (LuaValue::Nil, DomType::PhysicalProperties) => Ok(DomValue::PhysicalProperties(
                    dom::PhysicalProperties::Default,
                )),

                (LuaValue::UserData(u), d) => u.lua_to_dom_value(lua, Some(d)),

                (v, d) => Err(DomConversionError::ToDomValue {
                    to: d.variant_name().unwrap_or("???"),
                    from: v.type_name(),
                    detail: None,
                }),
            }
        } else {
            match self {
                LuaValue::Boolean(b) => Ok(DomValue::Bool(*b)),
                LuaValue::Integer(i) => Ok(DomValue::Int32(*i as i32)),
                LuaValue::Number(n) => Ok(DomValue::Float64(*n)),
                LuaValue::String(s) => Ok(DomValue::String(s.to_str()?.to_string())),
                LuaValue::UserData(u) => u.lua_to_dom_value(lua, None),
                v => Err(DomConversionError::ToDomValue {
                    to: "unknown",
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

macro_rules! dom_to_userdata {
    ($lua:expr, $value:ident => $to_type:ty) => {
        Ok($lua.create_userdata(Into::<$to_type>::into($value.clone()))?)
    };
}

/**
    Converts a generic lua userdata to an rbx-dom type.

    Since the type of the userdata needs to be specified
    in an explicit manner, this macro syntax was chosen:

    ```rs
    userdata_to_dom!(value_identifier as UserdataType => DomType)
    ```
*/
macro_rules! userdata_to_dom {
    ($userdata:ident as $from_type:ty => $to_type:ty) => {
        match $userdata.borrow::<$from_type>() {
            Ok(value) => Ok(From::<$to_type>::from(value.clone().into())),
            Err(error) => match error {
                LuaError::UserDataTypeMismatch => Err(DomConversionError::ToDomValue {
                    to: stringify!($to_type),
                    from: "userdata",
                    detail: Some("Type mismatch".to_string()),
                }),
                e => Err(DomConversionError::ToDomValue {
                    to: stringify!($to_type),
                    from: "userdata",
                    detail: Some(format!("Internal error: {e}")),
                }),
            },
        }
    };
}

impl DomValueToLua for LuaAnyUserData {
    #[rustfmt::skip]
    fn dom_value_to_lua(lua: &Lua, variant: &DomValue) -> DomConversionResult<Self> {
		use super::types::*;

        use rbx_dom_weak::types as dom;

        match variant {
            DomValue::Axes(value)           => dom_to_userdata!(lua, value => Axes),
            DomValue::BrickColor(value)     => dom_to_userdata!(lua, value => BrickColor),
            DomValue::CFrame(value)         => dom_to_userdata!(lua, value => CFrame),
            DomValue::Color3(value)         => dom_to_userdata!(lua, value => Color3),
            DomValue::Color3uint8(value)    => dom_to_userdata!(lua, value => Color3),
            DomValue::ColorSequence(value)  => dom_to_userdata!(lua, value => ColorSequence),
            DomValue::Content(value)        => dom_to_userdata!(lua, value => Content),
            DomValue::EnumItem(value)       => dom_to_userdata!(lua, value => EnumItem),
            DomValue::Faces(value)          => dom_to_userdata!(lua, value => Faces),
            DomValue::Font(value)           => dom_to_userdata!(lua, value => Font),
            DomValue::NumberRange(value)    => dom_to_userdata!(lua, value => NumberRange),
            DomValue::NumberSequence(value) => dom_to_userdata!(lua, value => NumberSequence),
            DomValue::Ray(value)            => dom_to_userdata!(lua, value => Ray),
            DomValue::Rect(value)           => dom_to_userdata!(lua, value => Rect),
            DomValue::Region3(value)        => dom_to_userdata!(lua, value => Region3),
            DomValue::Region3int16(value)   => dom_to_userdata!(lua, value => Region3int16),
            DomValue::UDim(value)           => dom_to_userdata!(lua, value => UDim),
            DomValue::UDim2(value)          => dom_to_userdata!(lua, value => UDim2),
            DomValue::UniqueId(value) => dom_to_userdata!(lua, value => UniqueId),
            DomValue::Vector2(value)        => dom_to_userdata!(lua, value => Vector2),
            DomValue::Vector2int16(value)   => dom_to_userdata!(lua, value => Vector2int16),
            DomValue::Vector3(value)        => dom_to_userdata!(lua, value => Vector3),
            DomValue::Vector3int16(value)   => dom_to_userdata!(lua, value => Vector3int16),

            // NOTE: The none and default variants of these types are handled in
			// DomValueToLua for the LuaValue type instead, allowing for nil/default
            DomValue::OptionalCFrame(Some(value)) => dom_to_userdata!(lua, value => CFrame),
            DomValue::PhysicalProperties(dom::PhysicalProperties::Custom(value)) => {
				dom_to_userdata!(lua, value => PhysicalProperties)
            },

            v => {
                Err(DomConversionError::FromDomValue {
                    from: v.variant_name().unwrap_or("???"),
                    to: "userdata",
                    detail: Some("Type not supported".to_string()),
                })
            }
        }
    }
}

impl LuaToDomValue for LuaAnyUserData {
    #[rustfmt::skip]
    fn lua_to_dom_value(&self, _: &Lua, variant_type: Option<DomType>) -> DomConversionResult<DomValue> {
        use super::types::*;

        use rbx_dom_weak::types as dom;

        if let Some(variant_type) = variant_type {
			/*
				Strict target type, use it to skip checking the actual
				type of the userdata and try to just do a pure conversion
			*/
            match variant_type {
                DomType::Axes           => userdata_to_dom!(self as Axes           => dom::Axes),
                DomType::BrickColor     => userdata_to_dom!(self as BrickColor     => dom::BrickColor),
                DomType::CFrame         => userdata_to_dom!(self as CFrame         => dom::CFrame),
                DomType::Color3         => userdata_to_dom!(self as Color3         => dom::Color3),
                DomType::Color3uint8    => userdata_to_dom!(self as Color3         => dom::Color3uint8),
                DomType::ColorSequence  => userdata_to_dom!(self as ColorSequence  => dom::ColorSequence),
                DomType::Content        => userdata_to_dom!(self as Content        => dom::Content),
                DomType::EnumItem       => userdata_to_dom!(self as EnumItem       => dom::EnumItem),
                DomType::Faces          => userdata_to_dom!(self as Faces          => dom::Faces),
                DomType::Font           => userdata_to_dom!(self as Font           => dom::Font),
                DomType::NumberRange    => userdata_to_dom!(self as NumberRange    => dom::NumberRange),
                DomType::NumberSequence => userdata_to_dom!(self as NumberSequence => dom::NumberSequence),
                DomType::Ray            => userdata_to_dom!(self as Ray            => dom::Ray),
                DomType::Rect           => userdata_to_dom!(self as Rect           => dom::Rect),
                DomType::Ref            => userdata_to_dom!(self as Instance       => dom::Ref),
                DomType::Region3        => userdata_to_dom!(self as Region3        => dom::Region3),
                DomType::Region3int16   => userdata_to_dom!(self as Region3int16   => dom::Region3int16),
                DomType::UDim           => userdata_to_dom!(self as UDim           => dom::UDim),
                DomType::UDim2          => userdata_to_dom!(self as UDim2          => dom::UDim2),
                DomType::UniqueId       => userdata_to_dom!(self as UniqueId       => dom::UniqueId),
                DomType::Vector2        => userdata_to_dom!(self as Vector2        => dom::Vector2),
                DomType::Vector2int16   => userdata_to_dom!(self as Vector2int16   => dom::Vector2int16),
                DomType::Vector3        => userdata_to_dom!(self as Vector3        => dom::Vector3),
                DomType::Vector3int16   => userdata_to_dom!(self as Vector3int16   => dom::Vector3int16),

	            // NOTE: The none and default variants of these types are handled in
				// LuaToDomValue for the LuaValue type instead, allowing for nil/default
                DomType::OptionalCFrame => {
                    match self.borrow::<CFrame>() {
                        Err(_) => unreachable!("Invalid use of conversion method, should be using LuaValue"),
                        Ok(value) => Ok(DomValue::OptionalCFrame(Some(dom::CFrame::from(*value)))),
                    }
                }
                DomType::PhysicalProperties => {
                    match self.borrow::<PhysicalProperties>() {
                        Err(_) => unreachable!("Invalid use of conversion method, should be using LuaValue"),
                        Ok(value) => {
                            let props = dom::CustomPhysicalProperties::from(*value);
                            let custom = dom::PhysicalProperties::Custom(props);
                            Ok(DomValue::PhysicalProperties(custom))
                        }
                    }
                }

                ty => {
                    Err(DomConversionError::ToDomValue {
                        to: ty.variant_name().unwrap_or("???"),
                        from: "userdata",
                        detail: Some("Type not supported".to_string()),
                    })
                }
            }
        } else {
			/*
				Non-strict target type, here we need to do manual typechecks
				on the userdata to see what we should be converting it into

				This is used for example for attributes, where the wanted
				type is not known by the dom and instead determined by the user
			*/
            match self {
                value if value.is::<Axes>()           => userdata_to_dom!(value as Axes           => dom::Axes),
                value if value.is::<BrickColor>()     => userdata_to_dom!(value as BrickColor     => dom::BrickColor),
                value if value.is::<CFrame>()         => userdata_to_dom!(value as CFrame         => dom::CFrame),
                value if value.is::<Color3>()         => userdata_to_dom!(value as Color3         => dom::Color3),
                value if value.is::<ColorSequence>()  => userdata_to_dom!(value as ColorSequence  => dom::ColorSequence),
                value if value.is::<EnumItem>()       => userdata_to_dom!(value as EnumItem       => dom::EnumItem),
                value if value.is::<Faces>()          => userdata_to_dom!(value as Faces          => dom::Faces),
                value if value.is::<Font>()           => userdata_to_dom!(value as Font           => dom::Font),
                value if value.is::<Instance>()       => userdata_to_dom!(value as Instance       => dom::Ref),
                value if value.is::<NumberRange>()    => userdata_to_dom!(value as NumberRange    => dom::NumberRange),
                value if value.is::<NumberSequence>() => userdata_to_dom!(value as NumberSequence => dom::NumberSequence),
                value if value.is::<Ray>()            => userdata_to_dom!(value as Ray            => dom::Ray),
                value if value.is::<Rect>()           => userdata_to_dom!(value as Rect           => dom::Rect),
                value if value.is::<Region3>()        => userdata_to_dom!(value as Region3        => dom::Region3),
                value if value.is::<Region3int16>()   => userdata_to_dom!(value as Region3int16   => dom::Region3int16),
                value if value.is::<UDim>()           => userdata_to_dom!(value as UDim           => dom::UDim),
                value if value.is::<UDim2>()          => userdata_to_dom!(value as UDim2          => dom::UDim2),
                value if value.is::<UniqueId>()       => userdata_to_dom!(value as UniqueId       => dom::UniqueId),
                value if value.is::<Vector2>()        => userdata_to_dom!(value as Vector2        => dom::Vector2),
                value if value.is::<Vector2int16>()   => userdata_to_dom!(value as Vector2int16   => dom::Vector2int16),
                value if value.is::<Vector3>()        => userdata_to_dom!(value as Vector3        => dom::Vector3),
                value if value.is::<Vector3int16>()   => userdata_to_dom!(value as Vector3int16   => dom::Vector3int16),

                _ => Err(DomConversionError::ToDomValue {
                    to: "unknown",
                    from: "userdata",
                    detail: Some("Type not supported".to_string()),
                })
            }
        }
    }
}
