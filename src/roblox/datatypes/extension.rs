use super::*;

pub(crate) trait DomValueExt {
    fn variant_name(&self) -> Option<&'static str>;
}

impl DomValueExt for DomType {
    fn variant_name(&self) -> Option<&'static str> {
        use DomType::*;
        Some(match self {
            Attributes => "Attributes",
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
            Font => "Font",
            Int32 => "Int32",
            Int64 => "Int64",
            MaterialColors => "MaterialColors",
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
            Tags => "Tags",
            UDim => "UDim",
            UDim2 => "UDim2",
            UniqueId => "UniqueId",
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
