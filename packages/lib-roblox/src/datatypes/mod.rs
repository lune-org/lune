use mlua::prelude::*;

pub(crate) use rbx_dom_weak::types::{Variant as RbxVariant, VariantType as RbxVariantType};

// NOTE: We create a new inner module scope here to make imports of datatypes more ergonomic

mod vector2;
mod vector3;

pub mod types {
    pub use super::vector2::Vector2;
    pub use super::vector3::Vector3;
}

// Trait definitions for conversion between rbx_dom_weak variant <-> our custom datatypes

#[allow(dead_code)]
pub(crate) enum RbxConversionError {
    FromRbxVariant {
        from: &'static str,
        to: &'static str,
        detail: Option<String>,
    },
    ToRbxVariant {
        to: &'static str,
        from: &'static str,
        detail: Option<String>,
    },
    DesiredTypeMismatch {
        can_convert_to: Option<&'static str>,
        detail: Option<String>,
    },
    External {
        message: String,
    },
}

impl RbxConversionError {
    pub fn external(e: impl std::error::Error) -> Self {
        RbxConversionError::External {
            message: e.to_string(),
        }
    }
}

pub(crate) type RbxConversionResult<T> = Result<T, RbxConversionError>;

pub(crate) trait ToRbxVariant {
    fn to_rbx_variant(
        &self,
        desired_type: Option<RbxVariantType>,
    ) -> RbxConversionResult<RbxVariant>;
}

pub(crate) trait FromRbxVariant: Sized {
    fn from_rbx_variant(variant: &RbxVariant) -> RbxConversionResult<Self>;
}

pub(crate) trait DatatypeTable {
    fn make_dt_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()>;
}

// Shared impls for datatype metamethods belonging to this module

fn datatype_impl_to_string<D>(_: &Lua, datatype: &D, _: ()) -> LuaResult<String>
where
    D: LuaUserData + ToString + 'static,
{
    Ok(datatype.to_string())
}

fn datatype_impl_eq<D>(_: &Lua, datatype: &D, value: LuaValue) -> LuaResult<bool>
where
    D: LuaUserData + PartialEq + 'static,
{
    if let LuaValue::UserData(ud) = value {
        if let Ok(vec) = ud.borrow::<D>() {
            Ok(*datatype == *vec)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

// NOTE: This implementation is .. not great, but it's the best we can
// do since we can't implement a trait like Display on a foreign type,
// and we are really only using it to make better error messages anyway

trait RbxVariantDisplayName {
    fn display_name(&self) -> &'static str;
}

impl RbxVariantDisplayName for RbxVariantType {
    fn display_name(&self) -> &'static str {
        use RbxVariantType::*;
        match self {
            Axes => "Axes",
            BinaryString => "BinaryString",
            Bool => "Bool",
            BrickColor => "BrickColor",
            CFrame => "CFrame",
            Color3 => "Color3",
            Color3uint8 => "Color3uint8",
            ColorSequence => "ColorSequence",
            Content => "Content",
            Enum => "Enum",
            Faces => "Faces",
            Float32 => "Float32",
            Float64 => "Float64",
            Int32 => "Int32",
            Int64 => "Int64",
            NumberRange => "NumberRange",
            NumberSequence => "NumberSequence",
            PhysicalProperties => "PhysicalProperties",
            Ray => "Ray",
            Rect => "Rect",
            Ref => "Ref",
            Region3 => "Region3",
            Region3int16 => "Region3int16",
            SharedString => "SharedString",
            String => "String",
            UDim => "UDim",
            UDim2 => "UDim2",
            Vector2 => "Vector2",
            Vector2int16 => "Vector2int16",
            Vector3 => "Vector3",
            Vector3int16 => "Vector3int16",
            OptionalCFrame => "OptionalCFrame",
            _ => "?",
        }
    }
}

impl RbxVariantDisplayName for RbxVariant {
    fn display_name(&self) -> &'static str {
        self.ty().display_name()
    }
}

// Generic impls for converting from lua values <-> rbx_dom_weak variants
// We use a separate trait here since creating lua stuff needs the lua context

pub(crate) trait FromRbxVariantLua<'lua>: Sized {
    fn from_rbx_variant_lua(variant: &RbxVariant, lua: &'lua Lua) -> RbxConversionResult<Self>;
}

impl<'lua> FromRbxVariantLua<'lua> for LuaValue<'lua> {
    fn from_rbx_variant_lua(variant: &RbxVariant, lua: &'lua Lua) -> RbxConversionResult<Self> {
        use self::types::*;
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::engine::Engine as _;
        use RbxVariant as Rbx;

        match variant {
            // Primitives
            Rbx::Bool(b) => Ok(LuaValue::Boolean(*b)),
            Rbx::Int64(i) => Ok(LuaValue::Integer(*i as i32)),
            Rbx::Int32(i) => Ok(LuaValue::Integer(*i)),
            Rbx::Float64(n) => Ok(LuaValue::Number(*n)),
            Rbx::Float32(n) => Ok(LuaValue::Number(*n as f64)),
            Rbx::String(s) => Ok(LuaValue::String(
                lua.create_string(s).map_err(RbxConversionError::external)?,
            )),
            Rbx::Content(s) => Ok(LuaValue::String(
                lua.create_string(AsRef::<str>::as_ref(s))
                    .map_err(RbxConversionError::external)?,
            )),
            Rbx::BinaryString(s) => {
                let encoded = STANDARD_NO_PAD.encode(AsRef::<[u8]>::as_ref(s));
                Ok(LuaValue::String(
                    lua.create_string(&encoded)
                        .map_err(RbxConversionError::external)?,
                ))
            }
            // Custom datatypes
            // NOTE: When adding a new datatype, also add it in the FromRbxVariantLua impl below
            Rbx::Vector2(_) => Vector2::from_rbx_variant(variant)?
                .to_lua(lua)
                .map_err(RbxConversionError::external),
            Rbx::Vector3(_) => Vector3::from_rbx_variant(variant)?
                .to_lua(lua)
                .map_err(RbxConversionError::external),
            // Not yet implemented datatypes
            Rbx::Axes(_) => todo!(),
            Rbx::BrickColor(_) => todo!(),
            Rbx::CFrame(_) => todo!(),
            Rbx::Color3(_) => todo!(),
            Rbx::Color3uint8(_) => todo!(),
            Rbx::ColorSequence(_) => todo!(),
            Rbx::Enum(_) => todo!(),
            Rbx::Faces(_) => todo!(),
            Rbx::NumberRange(_) => todo!(),
            Rbx::NumberSequence(_) => todo!(),
            Rbx::OptionalCFrame(_) => todo!(),
            Rbx::PhysicalProperties(_) => todo!(),
            Rbx::Ray(_) => todo!(),
            Rbx::Rect(_) => todo!(),
            Rbx::Region3(_) => todo!(),
            Rbx::Region3int16(_) => todo!(),
            Rbx::UDim(_) => todo!(),
            Rbx::UDim2(_) => todo!(),
            Rbx::Vector2int16(_) => todo!(),
            Rbx::Vector3int16(_) => todo!(),
            v => Err(RbxConversionError::FromRbxVariant {
                from: v.display_name(),
                to: "LuaValue",
                detail: Some("Type not supported".to_string()),
            }),
        }
    }
}

impl<'lua> ToRbxVariant for LuaValue<'lua> {
    fn to_rbx_variant(
        &self,
        desired_type: Option<RbxVariantType>,
    ) -> RbxConversionResult<RbxVariant> {
        use self::types::*;
        use base64::engine::general_purpose::STANDARD_NO_PAD;
        use base64::engine::Engine as _;
        use RbxVariantType as Rbx;

        if let Some(desired_type) = desired_type {
            match (self, desired_type) {
                // Primitives
                (LuaValue::Boolean(b), Rbx::Bool) => Ok(RbxVariant::Bool(*b)),
                (LuaValue::Integer(i), Rbx::Int64) => Ok(RbxVariant::Int64(*i as i64)),
                (LuaValue::Integer(i), Rbx::Int32) => Ok(RbxVariant::Int32(*i)),
                (LuaValue::Integer(i), Rbx::Float64) => Ok(RbxVariant::Float64(*i as f64)),
                (LuaValue::Integer(i), Rbx::Float32) => Ok(RbxVariant::Float32(*i as f32)),
                (LuaValue::Number(n), Rbx::Int64) => Ok(RbxVariant::Int64(*n as i64)),
                (LuaValue::Number(n), Rbx::Int32) => Ok(RbxVariant::Int32(*n as i32)),
                (LuaValue::Number(n), Rbx::Float64) => Ok(RbxVariant::Float64(*n)),
                (LuaValue::Number(n), Rbx::Float32) => Ok(RbxVariant::Float32(*n as f32)),
                (LuaValue::String(s), Rbx::String) => Ok(RbxVariant::String(
                    s.to_str()
                        .map_err(RbxConversionError::external)?
                        .to_string(),
                )),
                (LuaValue::String(s), Rbx::Content) => Ok(RbxVariant::Content(
                    s.to_str()
                        .map_err(RbxConversionError::external)?
                        .to_string()
                        .into(),
                )),
                (LuaValue::String(s), Rbx::BinaryString) => Ok(RbxVariant::BinaryString(
                    STANDARD_NO_PAD
                        .decode(s)
                        .map_err(RbxConversionError::external)?
                        .into(),
                )),
                // Custom datatypes
                // NOTE: When adding a new datatype, also add it below + in the FromRbxVariantLua impl above
                (LuaValue::UserData(u), d) => {
                    if let Ok(v2) = u.borrow::<Vector2>() {
                        v2.to_rbx_variant(Some(d))
                    } else if let Ok(v3) = u.borrow::<Vector3>() {
                        v3.to_rbx_variant(Some(d))
                    } else {
                        Err(RbxConversionError::ToRbxVariant {
                            to: d.display_name(),
                            from: "userdata",
                            detail: None,
                        })
                    }
                }
                // Not yet implemented rbx types
                (v, d) => Err(RbxConversionError::ToRbxVariant {
                    to: d.display_name(),
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
                LuaValue::String(s) => Ok(RbxVariant::String(
                    s.to_str()
                        .map_err(RbxConversionError::external)?
                        .to_string(),
                )),
                // Custom datatypes
                // NOTE: When adding a new datatype, also add it above
                LuaValue::UserData(u) => {
                    if let Ok(v2) = u.borrow::<Vector2>() {
                        v2.to_rbx_variant(None)
                    } else if let Ok(v3) = u.borrow::<Vector3>() {
                        v3.to_rbx_variant(None)
                    } else {
                        Err(RbxConversionError::ToRbxVariant {
                            to: "Variant",
                            from: "userdata",
                            detail: None,
                        })
                    }
                }
                // Not yet implemented rbx types
                v => Err(RbxConversionError::ToRbxVariant {
                    to: "Variant",
                    from: v.type_name(),
                    detail: None,
                }),
            }
        }
    }
}

// TODO: Implement tests for all datatypes in lua and run them here
// using the same mechanic we use to run tests in the main lib, these
// tests should also live next to other folders like fs, net, task, ..
