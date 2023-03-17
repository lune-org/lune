use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::{
    Font as RbxFont, FontStyle as RbxFontStyle, FontWeight as RbxFontWeight,
};

use super::{super::*, EnumItem};

/**
    An implementation of the [Font](https://create.roblox.com/docs/reference/engine/datatypes/Font) Roblox datatype.

    This implements all documented properties, methods & constructors of the Font class as of March 2023.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    family: String,
    weight: FontWeight,
    style: FontStyle,
}

impl Font {
    pub(crate) fn from_enum(material_enum_item: &EnumItem) -> Option<Font> {
        FONT_ENUM_MAP
            .iter()
            .find(|props| props.0 == material_enum_item.name && props.1.is_some())
            .map(|props| {
                let props = props.1.as_ref().unwrap().clone();
                Font {
                    family: props.0.to_string(),
                    weight: props.1,
                    style: props.2,
                }
            })
    }

    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        datatype_table.set(
            "fromEnum",
            lua.create_function(|_, value: EnumItem| {
                if value.parent.desc.name == "Font" {
                    match Font::from_enum(&value) {
                        Some(props) => Ok(props),
                        None => Err(LuaError::RuntimeError(format!(
                            "Found unknown Font '{}'",
                            value.name
                        ))),
                    }
                } else {
                    Err(LuaError::RuntimeError(format!(
                        "Expected argument #1 to be a Font, got {}",
                        value.parent.desc.name
                    )))
                }
            })?,
        )?;
        // TODO: Add fromName and fromId constructors
        // TODO: Add "new" constructor
        Ok(())
    }
}

impl LuaUserData for Font {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Family", |_, this| Ok(this.family.clone()));
        // TODO: Getters & setters for weight, style
        fields.add_field_method_get("Bold", |_, this| Ok(this.weight.clone().into_num() >= 600));
        fields.add_field_method_set("Bold", |_, this, value: bool| {
            if value {
                this.weight = FontWeight::Bold;
            } else {
                this.weight = FontWeight::Regular;
            }
            Ok(())
        });
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Font({}, {}, {})", self.family, self.weight, self.style)
    }
}

impl From<RbxFont> for Font {
    fn from(v: RbxFont) -> Self {
        Self {
            family: v.family,
            weight: v.weight.into(),
            style: v.style.into(),
        }
    }
}

impl From<Font> for RbxFont {
    fn from(v: Font) -> Self {
        RbxFont {
            family: v.family,
            weight: v.weight.into(),
            style: v.style.into(),
            cached_face_id: None,
        }
    }
}

impl From<RbxFontWeight> for FontWeight {
    fn from(v: RbxFontWeight) -> Self {
        FontWeight::from_num(v.as_u16()).expect("Missing font weight")
    }
}

impl From<FontWeight> for RbxFontWeight {
    fn from(v: FontWeight) -> Self {
        RbxFontWeight::from_u16(v.into_num()).expect("Missing rbx font weight")
    }
}

impl From<RbxFontStyle> for FontStyle {
    fn from(v: RbxFontStyle) -> Self {
        FontStyle::from_num(v.as_u8()).expect("Missing font weight")
    }
}

impl From<FontStyle> for RbxFontStyle {
    fn from(v: FontStyle) -> Self {
        RbxFontStyle::from_u8(v.into_num()).expect("Missing rbx font weight")
    }
}

/*

    NOTE: The font code below is all generated using the
    font_enum_map script in the scripts dir next to src,
    which can be ran in the Roblox Studio command bar

*/

type FontData = (&'static str, FontWeight, FontStyle);

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Heavy,
}

impl FontWeight {
    pub fn into_name(self) -> &'static str {
        match self {
            Self::Thin => "Thin",
            Self::ExtraLight => "ExtraLight",
            Self::Light => "Light",
            Self::Regular => "Regular",
            Self::Medium => "Medium",
            Self::SemiBold => "SemiBold",
            Self::Bold => "Bold",
            Self::ExtraBold => "ExtraBold",
            Self::Heavy => "Heavy",
        }
    }

    pub fn from_name(name: impl AsRef<str>) -> Option<Self> {
        match name.as_ref() {
            "Thin" => Some(Self::Thin),
            "ExtraLight" => Some(Self::ExtraLight),
            "Light" => Some(Self::Light),
            "Regular" => Some(Self::Regular),
            "Medium" => Some(Self::Medium),
            "SemiBold" => Some(Self::SemiBold),
            "Bold" => Some(Self::Bold),
            "ExtraBold" => Some(Self::ExtraBold),
            "Heavy" => Some(Self::Heavy),
            _ => None,
        }
    }

    pub fn into_num(self) -> u16 {
        match self {
            Self::Thin => 100,
            Self::ExtraLight => 200,
            Self::Light => 300,
            Self::Regular => 400,
            Self::Medium => 500,
            Self::SemiBold => 600,
            Self::Bold => 700,
            Self::ExtraBold => 800,
            Self::Heavy => 900,
        }
    }

    pub fn from_num(num: u16) -> Option<Self> {
        match num {
            100 => Some(Self::Thin),
            200 => Some(Self::ExtraLight),
            300 => Some(Self::Light),
            400 => Some(Self::Regular),
            500 => Some(Self::Medium),
            600 => Some(Self::SemiBold),
            700 => Some(Self::Bold),
            800 => Some(Self::ExtraBold),
            900 => Some(Self::Heavy),
            _ => None,
        }
    }
}

impl fmt::Display for FontWeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.clone().into_name())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FontStyle {
    Normal,
    Italic,
}

impl FontStyle {
    pub fn into_name(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Italic => "Italic",
        }
    }

    pub fn from_name(name: impl AsRef<str>) -> Option<Self> {
        match name.as_ref() {
            "Normal" => Some(Self::Normal),
            "Italic" => Some(Self::Italic),
            _ => None,
        }
    }

    pub fn into_num(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::Italic => 1,
        }
    }

    pub fn from_num(num: u8) -> Option<Self> {
        match num {
            0 => Some(Self::Normal),
            1 => Some(Self::Italic),
            _ => None,
        }
    }
}

impl fmt::Display for FontStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.clone().into_name())
    }
}

#[rustfmt::skip]
const FONT_ENUM_MAP: &[(&str, Option<FontData>)] = &[
    ("Legacy",             Some(("rbxasset://fonts/families/LegacyArial.json", FontWeight::Regular, FontStyle::Normal))),
    ("Arial",              Some(("rbxasset://fonts/families/Arial.json", FontWeight::Regular, FontStyle::Normal))),
    ("ArialBold",          Some(("rbxasset://fonts/families/Arial.json", FontWeight::Bold, FontStyle::Normal))),
    ("SourceSans",         Some(("rbxasset://fonts/families/SourceSansPro.json", FontWeight::Regular, FontStyle::Normal))),
    ("SourceSansBold",     Some(("rbxasset://fonts/families/SourceSansPro.json", FontWeight::Bold, FontStyle::Normal))),
    ("SourceSansSemibold", Some(("rbxasset://fonts/families/SourceSansPro.json", FontWeight::SemiBold, FontStyle::Normal))),
    ("SourceSansLight",    Some(("rbxasset://fonts/families/SourceSansPro.json", FontWeight::Light, FontStyle::Normal))),
    ("SourceSansItalic",   Some(("rbxasset://fonts/families/SourceSansPro.json", FontWeight::Regular, FontStyle::Italic))),
    ("Bodoni",             Some(("rbxasset://fonts/families/AccanthisADFStd.json", FontWeight::Regular, FontStyle::Normal))),
    ("Garamond",           Some(("rbxasset://fonts/families/Guru.json", FontWeight::Regular, FontStyle::Normal))),
    ("Cartoon",            Some(("rbxasset://fonts/families/ComicNeueAngular.json", FontWeight::Regular, FontStyle::Normal))),
    ("Code",               Some(("rbxasset://fonts/families/Inconsolata.json", FontWeight::Regular, FontStyle::Normal))),
    ("Highway",            Some(("rbxasset://fonts/families/HighwayGothic.json", FontWeight::Regular, FontStyle::Normal))),
    ("SciFi",              Some(("rbxasset://fonts/families/Zekton.json", FontWeight::Regular, FontStyle::Normal))),
    ("Arcade",             Some(("rbxasset://fonts/families/PressStart2P.json", FontWeight::Regular, FontStyle::Normal))),
    ("Fantasy",            Some(("rbxasset://fonts/families/Balthazar.json", FontWeight::Regular, FontStyle::Normal))),
    ("Antique",            Some(("rbxasset://fonts/families/RomanAntique.json", FontWeight::Regular, FontStyle::Normal))),
    ("Gotham",             Some(("rbxasset://fonts/families/GothamSSm.json", FontWeight::Regular, FontStyle::Normal))),
    ("GothamMedium",       Some(("rbxasset://fonts/families/GothamSSm.json", FontWeight::Medium, FontStyle::Normal))),
    ("GothamBold",         Some(("rbxasset://fonts/families/GothamSSm.json", FontWeight::Bold, FontStyle::Normal))),
    ("GothamBlack",        Some(("rbxasset://fonts/families/GothamSSm.json", FontWeight::Heavy, FontStyle::Normal))),
    ("AmaticSC",           Some(("rbxasset://fonts/families/AmaticSC.json", FontWeight::Regular, FontStyle::Normal))),
    ("Bangers",            Some(("rbxasset://fonts/families/Bangers.json", FontWeight::Regular, FontStyle::Normal))),
    ("Creepster",          Some(("rbxasset://fonts/families/Creepster.json", FontWeight::Regular, FontStyle::Normal))),
    ("DenkOne",            Some(("rbxasset://fonts/families/DenkOne.json", FontWeight::Regular, FontStyle::Normal))),
    ("Fondamento",         Some(("rbxasset://fonts/families/Fondamento.json", FontWeight::Regular, FontStyle::Normal))),
    ("FredokaOne",         Some(("rbxasset://fonts/families/FredokaOne.json", FontWeight::Regular, FontStyle::Normal))),
    ("GrenzeGotisch",      Some(("rbxasset://fonts/families/GrenzeGotisch.json", FontWeight::Regular, FontStyle::Normal))),
    ("IndieFlower",        Some(("rbxasset://fonts/families/IndieFlower.json", FontWeight::Regular, FontStyle::Normal))),
    ("JosefinSans",        Some(("rbxasset://fonts/families/JosefinSans.json", FontWeight::Regular, FontStyle::Normal))),
    ("Jura",               Some(("rbxasset://fonts/families/Jura.json", FontWeight::Regular, FontStyle::Normal))),
    ("Kalam",              Some(("rbxasset://fonts/families/Kalam.json", FontWeight::Regular, FontStyle::Normal))),
    ("LuckiestGuy",        Some(("rbxasset://fonts/families/LuckiestGuy.json", FontWeight::Regular, FontStyle::Normal))),
    ("Merriweather",       Some(("rbxasset://fonts/families/Merriweather.json", FontWeight::Regular, FontStyle::Normal))),
    ("Michroma",           Some(("rbxasset://fonts/families/Michroma.json", FontWeight::Regular, FontStyle::Normal))),
    ("Nunito",             Some(("rbxasset://fonts/families/Nunito.json", FontWeight::Regular, FontStyle::Normal))),
    ("Oswald",             Some(("rbxasset://fonts/families/Oswald.json", FontWeight::Regular, FontStyle::Normal))),
    ("PatrickHand",        Some(("rbxasset://fonts/families/PatrickHand.json", FontWeight::Regular, FontStyle::Normal))),
    ("PermanentMarker",    Some(("rbxasset://fonts/families/PermanentMarker.json", FontWeight::Regular, FontStyle::Normal))),
    ("Roboto",             Some(("rbxasset://fonts/families/Roboto.json", FontWeight::Regular, FontStyle::Normal))),
    ("RobotoCondensed",    Some(("rbxasset://fonts/families/RobotoCondensed.json", FontWeight::Regular, FontStyle::Normal))),
    ("RobotoMono",         Some(("rbxasset://fonts/families/RobotoMono.json", FontWeight::Regular, FontStyle::Normal))),
    ("Sarpanch",           Some(("rbxasset://fonts/families/Sarpanch.json", FontWeight::Regular, FontStyle::Normal))),
    ("SpecialElite",       Some(("rbxasset://fonts/families/SpecialElite.json", FontWeight::Regular, FontStyle::Normal))),
    ("TitilliumWeb",       Some(("rbxasset://fonts/families/TitilliumWeb.json", FontWeight::Regular, FontStyle::Normal))),
    ("Ubuntu",             Some(("rbxasset://fonts/families/Ubuntu.json", FontWeight::Regular, FontStyle::Normal))),
    ("Unknown", None),
];
