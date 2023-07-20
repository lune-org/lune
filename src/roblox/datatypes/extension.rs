use mlua::prelude::*;

use crate::roblox::instance::Instance;

use super::*;

pub(super) trait DomValueExt {
    fn variant_name(&self) -> Option<&'static str>;
}

impl DomValueExt for DomType {
    fn variant_name(&self) -> Option<&'static str> {
        use DomType::*;
        Some(match self {
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
            _ => return None,
        })
    }
}

impl DomValueExt for DomValue {
    fn variant_name(&self) -> Option<&'static str> {
        self.ty().variant_name()
    }
}

pub trait RobloxUserdataTypenameExt {
    fn roblox_type_name(&self) -> Option<&'static str>;
}

impl<'lua> RobloxUserdataTypenameExt for LuaAnyUserData<'lua> {
    #[rustfmt::skip]
    fn roblox_type_name(&self) -> Option<&'static str> {
        use super::types::*;

        Some(match self {
            value if value.is::<Axes>()                   => "Axes",
            value if value.is::<BrickColor>()             => "BrickColor",
            value if value.is::<CFrame>()                 => "CFrame",
            value if value.is::<Color3>()                 => "Color3",
            value if value.is::<ColorSequence>()          => "ColorSequence",
            value if value.is::<ColorSequenceKeypoint>()  => "ColorSequenceKeypoint",
            value if value.is::<Enums>()                  => "Enums",
            value if value.is::<Enum>()                   => "Enum",
            value if value.is::<EnumItem>()               => "EnumItem",
            value if value.is::<Faces>()                  => "Faces",
            value if value.is::<Font>()                   => "Font",
            value if value.is::<Instance>()               => "Instance",
            value if value.is::<NumberRange>()            => "NumberRange",
            value if value.is::<NumberSequence>()         => "NumberSequence",
            value if value.is::<NumberSequenceKeypoint>() => "NumberSequenceKeypoint",
            value if value.is::<PhysicalProperties>()     => "PhysicalProperties",
            value if value.is::<Ray>()                    => "Ray",
            value if value.is::<Rect>()                   => "Rect",
            value if value.is::<Region3>()                => "Region3",
            value if value.is::<Region3int16>()           => "Region3int16",
            value if value.is::<UDim>()                   => "UDim",
            value if value.is::<UDim2>()                  => "UDim2",
            value if value.is::<Vector2>()                => "Vector2",
            value if value.is::<Vector2int16>()           => "Vector2int16",
            value if value.is::<Vector3>()                => "Vector3",
            value if value.is::<Vector3int16>()           => "Vector3int16",
            _ => return None,
        })
    }
}
