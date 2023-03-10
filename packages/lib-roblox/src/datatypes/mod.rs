use mlua::prelude::*;

pub(crate) use rbx_dom_weak::types::{Variant as RbxVariant, VariantType as RbxVariantType};

// NOTE: We create a new inner module scope here to make imports of datatypes more ergonomic

mod vector3;

pub mod types {
    pub use super::vector3::Vector3;
}

// Trait definitions for conversion between rbx_dom_weak variant <-> datatype

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
        actual: &'static str,
        detail: Option<String>,
    },
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

// TODO: Implement tests for all datatypes in lua and run them here
// using the same mechanic we have to run tests in the main lib, these
// tests should also live next to other folders like fs, net, task, ..
