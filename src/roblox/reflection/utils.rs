use rbx_reflection::{ClassTag, DataType, PropertyTag, Scriptability};

use crate::roblox::datatypes::extension::DomValueExt;

pub fn data_type_to_str(data_type: DataType) -> String {
    match data_type {
        DataType::Enum(e) => format!("Enum.{e}"),
        DataType::Value(v) => v
            .variant_name()
            .expect("Encountered unknown data type variant")
            .to_string(),
        _ => panic!("Encountered unknown data type"),
    }
}

/*
     NOTE: Remember to add any new strings here to typedefs too!
*/

pub fn scriptability_to_str(scriptability: &Scriptability) -> &'static str {
    match scriptability {
        Scriptability::None => "None",
        Scriptability::Custom => "Custom",
        Scriptability::Read => "Read",
        Scriptability::ReadWrite => "ReadWrite",
        Scriptability::Write => "Write",
        _ => panic!("Encountered unknown scriptability"),
    }
}

pub fn property_tag_to_str(tag: &PropertyTag) -> &'static str {
    match tag {
        PropertyTag::Deprecated => "Deprecated",
        PropertyTag::Hidden => "Hidden",
        PropertyTag::NotBrowsable => "NotBrowsable",
        PropertyTag::NotReplicated => "NotReplicated",
        PropertyTag::NotScriptable => "NotScriptable",
        PropertyTag::ReadOnly => "ReadOnly",
        PropertyTag::WriteOnly => "WriteOnly",
        _ => panic!("Encountered unknown property tag"),
    }
}

pub fn class_tag_to_str(tag: &ClassTag) -> &'static str {
    match tag {
        ClassTag::Deprecated => "Deprecated",
        ClassTag::NotBrowsable => "NotBrowsable",
        ClassTag::NotCreatable => "NotCreatable",
        ClassTag::NotReplicated => "NotReplicated",
        ClassTag::PlayerReplicated => "PlayerReplicated",
        ClassTag::Service => "Service",
        ClassTag::Settings => "Settings",
        ClassTag::UserSettings => "UserSettings",
        _ => panic!("Encountered unknown class tag"),
    }
}
