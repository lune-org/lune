use super::*;

pub(crate) trait DomValueExt {
    fn variant_name(&self) -> &'static str;
}

impl DomValueExt for DomType {
    fn variant_name(&self) -> &'static str {
        use DomType::*;
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

impl DomValueExt for DomValue {
    fn variant_name(&self) -> &'static str {
        self.ty().variant_name()
    }
}
