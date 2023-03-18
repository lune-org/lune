use core::fmt;
use std::str::FromStr;

use mlua::prelude::*;
use rbx_dom_weak::types::{
    Font as DomFont, FontStyle as DomFontStyle, FontWeight as DomFontWeight,
};

use super::{super::*, EnumItem};

/**
    An implementation of the [Font](https://create.roblox.com/docs/reference/engine/datatypes/Font) Roblox datatype.

    This implements all documented properties, methods & constructors of the Font class as of March 2023.
*/
#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    pub(crate) family: String,
    pub(crate) weight: FontWeight,
    pub(crate) style: FontStyle,
    pub(crate) cached_id: Option<String>,
}

impl Font {
    pub(crate) fn from_enum_item(material_enum_item: &EnumItem) -> Option<Font> {
        FONT_ENUM_MAP
            .iter()
            .find(|props| props.0 == material_enum_item.name && props.1.is_some())
            .map(|props| props.1.as_ref().unwrap())
            .map(|props| Font {
                family: props.0.to_string(),
                weight: props.1,
                style: props.2,
                cached_id: None,
            })
    }

    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        datatype_table.set(
            "new",
            lua.create_function(
                |_, (family, weight, style): (String, Option<FontWeight>, Option<FontStyle>)| {
                    Ok(Font {
                        family,
                        weight: weight.unwrap_or_default(),
                        style: style.unwrap_or_default(),
                        cached_id: None,
                    })
                },
            )?,
        )?;
        datatype_table.set(
            "fromEnum",
            lua.create_function(|_, value: EnumItem| {
                if value.parent.desc.name == "Font" {
                    match Font::from_enum_item(&value) {
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
        datatype_table.set(
            "fromName",
            lua.create_function(
                |_, (file, weight, style): (String, Option<FontWeight>, Option<FontStyle>)| {
                    Ok(Font {
                        family: format!("rbxasset://fonts/families/{}.json", file),
                        weight: weight.unwrap_or_default(),
                        style: style.unwrap_or_default(),
                        cached_id: None,
                    })
                },
            )?,
        )?;
        datatype_table.set(
            "fromId",
            lua.create_function(
                |_, (id, weight, style): (i32, Option<FontWeight>, Option<FontStyle>)| {
                    Ok(Font {
                        family: format!("rbxassetid://{}", id),
                        weight: weight.unwrap_or_default(),
                        style: style.unwrap_or_default(),
                        cached_id: None,
                    })
                },
            )?,
        )
    }
}

impl LuaUserData for Font {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        // Getters
        fields.add_field_method_get("Family", |_, this| Ok(this.family.clone()));
        fields.add_field_method_get("Weight", |_, this| Ok(this.weight));
        fields.add_field_method_get("Style", |_, this| Ok(this.style));
        fields.add_field_method_get("Bold", |_, this| Ok(this.weight.as_u16() >= 600));
        // Setters
        fields.add_field_method_set("Weight", |_, this, value: FontWeight| {
            this.weight = value;
            Ok(())
        });
        fields.add_field_method_set("Style", |_, this, value: FontStyle| {
            this.style = value;
            Ok(())
        });
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
        write!(f, "{}, {}, {}", self.family, self.weight, self.style)
    }
}

impl From<DomFont> for Font {
    fn from(v: DomFont) -> Self {
        Self {
            family: v.family,
            weight: v.weight.into(),
            style: v.style.into(),
            cached_id: v.cached_face_id,
        }
    }
}

impl From<Font> for DomFont {
    fn from(v: Font) -> Self {
        DomFont {
            family: v.family,
            weight: v.weight.into(),
            style: v.style.into(),
            cached_face_id: v.cached_id,
        }
    }
}

impl From<DomFontWeight> for FontWeight {
    fn from(v: DomFontWeight) -> Self {
        FontWeight::from_u16(v.as_u16()).expect("Missing font weight")
    }
}

impl From<FontWeight> for DomFontWeight {
    fn from(v: FontWeight) -> Self {
        DomFontWeight::from_u16(v.as_u16()).expect("Missing rbx font weight")
    }
}

impl From<DomFontStyle> for FontStyle {
    fn from(v: DomFontStyle) -> Self {
        FontStyle::from_u8(v.as_u8()).expect("Missing font weight")
    }
}

impl From<FontStyle> for DomFontStyle {
    fn from(v: FontStyle) -> Self {
        DomFontStyle::from_u8(v.as_u8()).expect("Missing rbx font weight")
    }
}

/*

    NOTE: The font code below is all generated using the
    font_enum_map script in the scripts dir next to src,
    which can be ran in the Roblox Studio command bar

*/

type FontData = (&'static str, FontWeight, FontStyle);

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub(crate) fn as_u16(&self) -> u16 {
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

    pub(crate) fn from_u16(n: u16) -> Option<Self> {
        match n {
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

impl Default for FontWeight {
    fn default() -> Self {
        Self::Regular
    }
}

impl std::str::FromStr for FontWeight {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Thin" => Ok(Self::Thin),
            "ExtraLight" => Ok(Self::ExtraLight),
            "Light" => Ok(Self::Light),
            "Regular" => Ok(Self::Regular),
            "Medium" => Ok(Self::Medium),
            "SemiBold" => Ok(Self::SemiBold),
            "Bold" => Ok(Self::Bold),
            "ExtraBold" => Ok(Self::ExtraBold),
            "Heavy" => Ok(Self::Heavy),
            _ => Err("Unknown FontWeight"),
        }
    }
}

impl std::fmt::Display for FontWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
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
        )
    }
}

impl<'lua> FromLua<'lua> for FontWeight {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        let mut message = None;
        if let LuaValue::UserData(ud) = &lua_value {
            let value = ud.borrow::<EnumItem>()?;
            if value.parent.desc.name == "FontWeight" {
                if let Ok(value) = FontWeight::from_str(&value.name) {
                    return Ok(value);
                } else {
                    message = Some(format!(
                        "Found unknown Enum.FontWeight value '{}'",
                        value.name
                    ));
                }
            } else {
                message = Some(format!(
                    "Expected Enum.FontWeight, got Enum.{}",
                    value.parent.desc.name
                ));
            }
        }
        Err(LuaError::FromLuaConversionError {
            from: lua_value.type_name(),
            to: "Enum.FontWeight",
            message,
        })
    }
}

impl<'lua> ToLua<'lua> for FontWeight {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        match EnumItem::from_enum_name_and_name("FontWeight", self.to_string()) {
            Some(enum_item) => Ok(LuaValue::UserData(lua.create_userdata(enum_item)?)),
            None => Err(LuaError::ToLuaConversionError {
                from: "FontWeight",
                to: "EnumItem",
                message: Some(format!("Found unknown Enum.FontWeight value '{}'", self)),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FontStyle {
    Normal,
    Italic,
}

impl FontStyle {
    pub(crate) fn as_u8(&self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::Italic => 1,
        }
    }

    pub(crate) fn from_u8(n: u8) -> Option<Self> {
        match n {
            0 => Some(Self::Normal),
            1 => Some(Self::Italic),
            _ => None,
        }
    }
}

impl Default for FontStyle {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::str::FromStr for FontStyle {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Normal" => Ok(Self::Normal),
            "Italic" => Ok(Self::Italic),
            _ => Err("Unknown FontStyle"),
        }
    }
}

impl std::fmt::Display for FontStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Normal => "Normal",
                Self::Italic => "Italic",
            }
        )
    }
}

impl<'lua> FromLua<'lua> for FontStyle {
    fn from_lua(lua_value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        let mut message = None;
        if let LuaValue::UserData(ud) = &lua_value {
            let value = ud.borrow::<EnumItem>()?;
            if value.parent.desc.name == "FontStyle" {
                if let Ok(value) = FontStyle::from_str(&value.name) {
                    return Ok(value);
                } else {
                    message = Some(format!(
                        "Found unknown Enum.FontStyle value '{}'",
                        value.name
                    ));
                }
            } else {
                message = Some(format!(
                    "Expected Enum.FontStyle, got Enum.{}",
                    value.parent.desc.name
                ));
            }
        }
        Err(LuaError::FromLuaConversionError {
            from: lua_value.type_name(),
            to: "Enum.FontStyle",
            message,
        })
    }
}

impl<'lua> ToLua<'lua> for FontStyle {
    fn to_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        match EnumItem::from_enum_name_and_name("FontStyle", self.to_string()) {
            Some(enum_item) => Ok(LuaValue::UserData(lua.create_userdata(enum_item)?)),
            None => Err(LuaError::ToLuaConversionError {
                from: "FontStyle",
                to: "EnumItem",
                message: Some(format!("Found unknown Enum.FontStyle value '{}'", self)),
            }),
        }
    }
}

#[rustfmt::skip]
const FONT_ENUM_MAP: &[(&str, Option<FontData>)] = &[
    ("Legacy",             Some(("rbxasset://fonts/families/LegacyArial.json",      FontWeight::Regular,  FontStyle::Normal))),
    ("Arial",              Some(("rbxasset://fonts/families/Arial.json",            FontWeight::Regular,  FontStyle::Normal))),
    ("ArialBold",          Some(("rbxasset://fonts/families/Arial.json",            FontWeight::Bold,     FontStyle::Normal))),
    ("SourceSans",         Some(("rbxasset://fonts/families/SourceSansPro.json",    FontWeight::Regular,  FontStyle::Normal))),
    ("SourceSansBold",     Some(("rbxasset://fonts/families/SourceSansPro.json",    FontWeight::Bold,     FontStyle::Normal))),
    ("SourceSansSemibold", Some(("rbxasset://fonts/families/SourceSansPro.json",    FontWeight::SemiBold, FontStyle::Normal))),
    ("SourceSansLight",    Some(("rbxasset://fonts/families/SourceSansPro.json",    FontWeight::Light,    FontStyle::Normal))),
    ("SourceSansItalic",   Some(("rbxasset://fonts/families/SourceSansPro.json",    FontWeight::Regular,  FontStyle::Italic))),
    ("Bodoni",             Some(("rbxasset://fonts/families/AccanthisADFStd.json",  FontWeight::Regular,  FontStyle::Normal))),
    ("Garamond",           Some(("rbxasset://fonts/families/Guru.json",             FontWeight::Regular,  FontStyle::Normal))),
    ("Cartoon",            Some(("rbxasset://fonts/families/ComicNeueAngular.json", FontWeight::Regular,  FontStyle::Normal))),
    ("Code",               Some(("rbxasset://fonts/families/Inconsolata.json",      FontWeight::Regular,  FontStyle::Normal))),
    ("Highway",            Some(("rbxasset://fonts/families/HighwayGothic.json",    FontWeight::Regular,  FontStyle::Normal))),
    ("SciFi",              Some(("rbxasset://fonts/families/Zekton.json",           FontWeight::Regular,  FontStyle::Normal))),
    ("Arcade",             Some(("rbxasset://fonts/families/PressStart2P.json",     FontWeight::Regular,  FontStyle::Normal))),
    ("Fantasy",            Some(("rbxasset://fonts/families/Balthazar.json",        FontWeight::Regular,  FontStyle::Normal))),
    ("Antique",            Some(("rbxasset://fonts/families/RomanAntique.json",     FontWeight::Regular,  FontStyle::Normal))),
    ("Gotham",             Some(("rbxasset://fonts/families/GothamSSm.json",        FontWeight::Regular,  FontStyle::Normal))),
    ("GothamMedium",       Some(("rbxasset://fonts/families/GothamSSm.json",        FontWeight::Medium,   FontStyle::Normal))),
    ("GothamBold",         Some(("rbxasset://fonts/families/GothamSSm.json",        FontWeight::Bold,     FontStyle::Normal))),
    ("GothamBlack",        Some(("rbxasset://fonts/families/GothamSSm.json",        FontWeight::Heavy,    FontStyle::Normal))),
    ("AmaticSC",           Some(("rbxasset://fonts/families/AmaticSC.json",         FontWeight::Regular,  FontStyle::Normal))),
    ("Bangers",            Some(("rbxasset://fonts/families/Bangers.json",          FontWeight::Regular,  FontStyle::Normal))),
    ("Creepster",          Some(("rbxasset://fonts/families/Creepster.json",        FontWeight::Regular,  FontStyle::Normal))),
    ("DenkOne",            Some(("rbxasset://fonts/families/DenkOne.json",          FontWeight::Regular,  FontStyle::Normal))),
    ("Fondamento",         Some(("rbxasset://fonts/families/Fondamento.json",       FontWeight::Regular,  FontStyle::Normal))),
    ("FredokaOne",         Some(("rbxasset://fonts/families/FredokaOne.json",       FontWeight::Regular,  FontStyle::Normal))),
    ("GrenzeGotisch",      Some(("rbxasset://fonts/families/GrenzeGotisch.json",    FontWeight::Regular,  FontStyle::Normal))),
    ("IndieFlower",        Some(("rbxasset://fonts/families/IndieFlower.json",      FontWeight::Regular,  FontStyle::Normal))),
    ("JosefinSans",        Some(("rbxasset://fonts/families/JosefinSans.json",      FontWeight::Regular,  FontStyle::Normal))),
    ("Jura",               Some(("rbxasset://fonts/families/Jura.json",             FontWeight::Regular,  FontStyle::Normal))),
    ("Kalam",              Some(("rbxasset://fonts/families/Kalam.json",            FontWeight::Regular,  FontStyle::Normal))),
    ("LuckiestGuy",        Some(("rbxasset://fonts/families/LuckiestGuy.json",      FontWeight::Regular,  FontStyle::Normal))),
    ("Merriweather",       Some(("rbxasset://fonts/families/Merriweather.json",     FontWeight::Regular,  FontStyle::Normal))),
    ("Michroma",           Some(("rbxasset://fonts/families/Michroma.json",         FontWeight::Regular,  FontStyle::Normal))),
    ("Nunito",             Some(("rbxasset://fonts/families/Nunito.json",           FontWeight::Regular,  FontStyle::Normal))),
    ("Oswald",             Some(("rbxasset://fonts/families/Oswald.json",           FontWeight::Regular,  FontStyle::Normal))),
    ("PatrickHand",        Some(("rbxasset://fonts/families/PatrickHand.json",      FontWeight::Regular,  FontStyle::Normal))),
    ("PermanentMarker",    Some(("rbxasset://fonts/families/PermanentMarker.json",  FontWeight::Regular,  FontStyle::Normal))),
    ("Roboto",             Some(("rbxasset://fonts/families/Roboto.json",           FontWeight::Regular,  FontStyle::Normal))),
    ("RobotoCondensed",    Some(("rbxasset://fonts/families/RobotoCondensed.json",  FontWeight::Regular,  FontStyle::Normal))),
    ("RobotoMono",         Some(("rbxasset://fonts/families/RobotoMono.json",       FontWeight::Regular,  FontStyle::Normal))),
    ("Sarpanch",           Some(("rbxasset://fonts/families/Sarpanch.json",         FontWeight::Regular,  FontStyle::Normal))),
    ("SpecialElite",       Some(("rbxasset://fonts/families/SpecialElite.json",     FontWeight::Regular,  FontStyle::Normal))),
    ("TitilliumWeb",       Some(("rbxasset://fonts/families/TitilliumWeb.json",     FontWeight::Regular,  FontStyle::Normal))),
    ("Ubuntu",             Some(("rbxasset://fonts/families/Ubuntu.json",           FontWeight::Regular,  FontStyle::Normal))),
    ("Unknown",            None),
];
