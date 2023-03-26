use mlua::prelude::*;

use rbx_dom_weak::types::{Variant as DomValue, VariantType as DomType};

use super::extension::DomValueExt;

pub fn ensure_valid_attribute_name(name: impl AsRef<str>) -> LuaResult<()> {
    let name = name.as_ref();
    if name.to_ascii_uppercase().starts_with("RBX") {
        Err(LuaError::RuntimeError(
            "Attribute names must not start with the prefix \"RBX\"".to_string(),
        ))
    } else if !name.chars().all(|c| c == '_' || c.is_alphanumeric()) {
        Err(LuaError::RuntimeError(
            "Attribute names must only use alphanumeric characters and underscore".to_string(),
        ))
    } else if name.len() > 100 {
        Err(LuaError::RuntimeError(
            "Attribute names must be 100 characters or less in length".to_string(),
        ))
    } else {
        Ok(())
    }
}

pub fn ensure_valid_attribute_value(value: &DomValue) -> LuaResult<()> {
    // TODO: Uncomment supported types when they are also supported in rbx-dom serialization
    let is_valid = matches!(
        value.ty(),
        DomType::Bool
            | DomType::BrickColor
            // | DomType::CFrame
            | DomType::Color3
            | DomType::ColorSequence
            | DomType::Float32
            | DomType::Float64
            | DomType::Int32
            | DomType::Int64
            | DomType::NumberRange
            | DomType::NumberSequence
            | DomType::Rect
            | DomType::String
            | DomType::UDim
            | DomType::UDim2
            | DomType::Vector2
            | DomType::Vector3 // | DomType::Font
    );
    if is_valid {
        Ok(())
    } else {
        Err(LuaError::RuntimeError(format!(
            "'{}' is not a valid attribute type",
            value.ty().variant_name()
        )))
    }
}
