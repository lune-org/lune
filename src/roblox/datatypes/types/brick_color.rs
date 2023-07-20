use core::fmt;

use mlua::prelude::*;
use rand::seq::SliceRandom;
use rbx_dom_weak::types::BrickColor as DomBrickColor;

use super::{super::*, Color3};

/**
    An implementation of the [BrickColor](https://create.roblox.com/docs/reference/engine/datatypes/BrickColor) Roblox datatype.

    This implements all documented properties, methods & constructors of the BrickColor class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrickColor {
    // Unfortunately we can't use DomBrickColor as the backing type here
    // because it does not expose any way of getting the actual rgb colors :-(
    pub(crate) number: u16,
    pub(crate) name: &'static str,
    pub(crate) rgb: (u8, u8, u8),
}

impl BrickColor {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        type ArgsNumber = u16;
        type ArgsName = String;
        type ArgsRgb = (u8, u8, u8);
        type ArgsColor3<'lua> = LuaUserDataRef<'lua, Color3>;
        datatype_table.set(
            "new",
            lua.create_function(|lua, args: LuaMultiValue| {
                if let Ok(number) = ArgsNumber::from_lua_multi(args.clone(), lua) {
                    Ok(color_from_number(number))
                } else if let Ok(name) = ArgsName::from_lua_multi(args.clone(), lua) {
                    Ok(color_from_name(name))
                } else if let Ok((r, g, b)) = ArgsRgb::from_lua_multi(args.clone(), lua) {
                    Ok(color_from_rgb(r, g, b))
                } else if let Ok(color) = ArgsColor3::from_lua_multi(args.clone(), lua) {
                    Ok(Self::from(*color))
                } else {
                    // FUTURE: Better error message here using given arg types
                    Err(LuaError::RuntimeError(
                        "Invalid arguments to constructor".to_string(),
                    ))
                }
            })?,
        )?;
        datatype_table.set(
            "palette",
            lua.create_function(|_, index: u16| {
                if index == 0 {
                    Err(LuaError::RuntimeError("Invalid index".to_string()))
                } else if let Some(number) = BRICK_COLOR_PALETTE.get((index - 1) as usize) {
                    Ok(color_from_number(*number))
                } else {
                    Err(LuaError::RuntimeError("Invalid index".to_string()))
                }
            })?,
        )?;
        datatype_table.set(
            "random",
            lua.create_function(|_, ()| {
                let number = BRICK_COLOR_PALETTE.choose(&mut rand::thread_rng());
                Ok(color_from_number(*number.unwrap()))
            })?,
        )?;
        for (name, number) in BRICK_COLOR_CONSTRUCTORS {
            datatype_table.set(
                *name,
                lua.create_function(|_, ()| Ok(color_from_number(*number)))?,
            )?;
        }
        Ok(())
    }
}

impl LuaUserData for BrickColor {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Number", |_, this| Ok(this.number));
        fields.add_field_method_get("Name", |_, this| Ok(this.name));
        fields.add_field_method_get("R", |_, this| Ok(this.rgb.0 as f32 / 255f32));
        fields.add_field_method_get("G", |_, this| Ok(this.rgb.1 as f32 / 255f32));
        fields.add_field_method_get("B", |_, this| Ok(this.rgb.2 as f32 / 255f32));
        fields.add_field_method_get("r", |_, this| Ok(this.rgb.0 as f32 / 255f32));
        fields.add_field_method_get("g", |_, this| Ok(this.rgb.1 as f32 / 255f32));
        fields.add_field_method_get("b", |_, this| Ok(this.rgb.2 as f32 / 255f32));
        fields.add_field_method_get("Color", |_, this| Ok(Color3::from(*this)));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl Default for BrickColor {
    fn default() -> Self {
        color_from_number(BRICK_COLOR_DEFAULT)
    }
}

impl fmt::Display for BrickColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<Color3> for BrickColor {
    fn from(value: Color3) -> Self {
        let r = value.r.clamp(u8::MIN as f32, u8::MAX as f32) as u8;
        let g = value.g.clamp(u8::MIN as f32, u8::MAX as f32) as u8;
        let b = value.b.clamp(u8::MIN as f32, u8::MAX as f32) as u8;
        color_from_rgb(r, g, b)
    }
}

impl From<BrickColor> for Color3 {
    fn from(value: BrickColor) -> Self {
        Self {
            r: (value.rgb.0 as f32) / 255.0,
            g: (value.rgb.1 as f32) / 255.0,
            b: (value.rgb.2 as f32) / 255.0,
        }
    }
}

impl From<DomBrickColor> for BrickColor {
    fn from(v: DomBrickColor) -> Self {
        color_from_name(v.to_string())
    }
}

impl From<BrickColor> for DomBrickColor {
    fn from(v: BrickColor) -> Self {
        DomBrickColor::from_number(v.number).unwrap_or(DomBrickColor::MediumStoneGrey)
    }
}

/*

    NOTE: The brick color definitions below are generated using
    the brick_color script in the scripts dir next to src, which can
    be ran using `cargo run packages/lib-roblox/scripts/brick_color`

*/

type BrickColorDef = &'static (u16, &'static str, (u8, u8, u8));

impl From<BrickColorDef> for BrickColor {
    fn from(value: BrickColorDef) -> Self {
        Self {
            number: value.0,
            name: value.1,
            rgb: value.2,
        }
    }
}

const BRICK_COLOR_DEFAULT_VALUE: BrickColorDef =
    &BRICK_COLOR_VALUES[(BRICK_COLOR_DEFAULT - 1) as usize];

fn color_from_number(index: u16) -> BrickColor {
    BRICK_COLOR_VALUES
        .iter()
        .find(|color| color.0 == index)
        .unwrap_or(BRICK_COLOR_DEFAULT_VALUE)
        .into()
}

fn color_from_name(name: impl AsRef<str>) -> BrickColor {
    let name = name.as_ref();
    BRICK_COLOR_VALUES
        .iter()
        .find(|color| color.1 == name)
        .unwrap_or(BRICK_COLOR_DEFAULT_VALUE)
        .into()
}

fn color_from_rgb(r: u8, g: u8, b: u8) -> BrickColor {
    let r = r as i16;
    let g = g as i16;
    let b = b as i16;
    BRICK_COLOR_VALUES
        .iter()
        .fold(
            (None, u16::MAX),
            |(closest_color, closest_distance), color| {
                let cr = color.2 .0 as i16;
                let cg = color.2 .1 as i16;
                let cb = color.2 .2 as i16;
                let distance = ((r - cr) + (g - cg) + (b - cb)).unsigned_abs();
                if distance < closest_distance {
                    (Some(color), distance)
                } else {
                    (closest_color, closest_distance)
                }
            },
        )
        .0
        .unwrap_or(BRICK_COLOR_DEFAULT_VALUE)
        .into()
}

const BRICK_COLOR_DEFAULT: u16 = 194;

const BRICK_COLOR_VALUES: &[(u16, &str, (u8, u8, u8))] = &[
    (1, "White", (242, 243, 243)),
    (2, "Grey", (161, 165, 162)),
    (3, "Light yellow", (249, 233, 153)),
    (5, "Brick yellow", (215, 197, 154)),
    (6, "Light green (Mint)", (194, 218, 184)),
    (9, "Light reddish violet", (232, 186, 200)),
    (11, "Pastel Blue", (128, 187, 219)),
    (12, "Light orange brown", (203, 132, 66)),
    (18, "Nougat", (204, 142, 105)),
    (21, "Bright red", (196, 40, 28)),
    (22, "Med. reddish violet", (196, 112, 160)),
    (23, "Bright blue", (13, 105, 172)),
    (24, "Bright yellow", (245, 205, 48)),
    (25, "Earth orange", (98, 71, 50)),
    (26, "Black", (27, 42, 53)),
    (27, "Dark grey", (109, 110, 108)),
    (28, "Dark green", (40, 127, 71)),
    (29, "Medium green", (161, 196, 140)),
    (36, "Lig. Yellowich orange", (243, 207, 155)),
    (37, "Bright green", (75, 151, 75)),
    (38, "Dark orange", (160, 95, 53)),
    (39, "Light bluish violet", (193, 202, 222)),
    (40, "Transparent", (236, 236, 236)),
    (41, "Tr. Red", (205, 84, 75)),
    (42, "Tr. Lg blue", (193, 223, 240)),
    (43, "Tr. Blue", (123, 182, 232)),
    (44, "Tr. Yellow", (247, 241, 141)),
    (45, "Light blue", (180, 210, 228)),
    (47, "Tr. Flu. Reddish orange", (217, 133, 108)),
    (48, "Tr. Green", (132, 182, 141)),
    (49, "Tr. Flu. Green", (248, 241, 132)),
    (50, "Phosph. White", (236, 232, 222)),
    (100, "Light red", (238, 196, 182)),
    (101, "Medium red", (218, 134, 122)),
    (102, "Medium blue", (110, 153, 202)),
    (103, "Light grey", (199, 193, 183)),
    (104, "Bright violet", (107, 50, 124)),
    (105, "Br. yellowish orange", (226, 155, 64)),
    (106, "Bright orange", (218, 133, 65)),
    (107, "Bright bluish green", (0, 143, 156)),
    (108, "Earth yellow", (104, 92, 67)),
    (110, "Bright bluish violet", (67, 84, 147)),
    (111, "Tr. Brown", (191, 183, 177)),
    (112, "Medium bluish violet", (104, 116, 172)),
    (113, "Tr. Medi. reddish violet", (229, 173, 200)),
    (115, "Med. yellowish green", (199, 210, 60)),
    (116, "Med. bluish green", (85, 165, 175)),
    (118, "Light bluish green", (183, 215, 213)),
    (119, "Br. yellowish green", (164, 189, 71)),
    (120, "Lig. yellowish green", (217, 228, 167)),
    (121, "Med. yellowish orange", (231, 172, 88)),
    (123, "Br. reddish orange", (211, 111, 76)),
    (124, "Bright reddish violet", (146, 57, 120)),
    (125, "Light orange", (234, 184, 146)),
    (126, "Tr. Bright bluish violet", (165, 165, 203)),
    (127, "Gold", (220, 188, 129)),
    (128, "Dark nougat", (174, 122, 89)),
    (131, "Silver", (156, 163, 168)),
    (133, "Neon orange", (213, 115, 61)),
    (134, "Neon green", (216, 221, 86)),
    (135, "Sand blue", (116, 134, 157)),
    (136, "Sand violet", (135, 124, 144)),
    (137, "Medium orange", (224, 152, 100)),
    (138, "Sand yellow", (149, 138, 115)),
    (140, "Earth blue", (32, 58, 86)),
    (141, "Earth green", (39, 70, 45)),
    (143, "Tr. Flu. Blue", (207, 226, 247)),
    (145, "Sand blue metallic", (121, 136, 161)),
    (146, "Sand violet metallic", (149, 142, 163)),
    (147, "Sand yellow metallic", (147, 135, 103)),
    (148, "Dark grey metallic", (87, 88, 87)),
    (149, "Black metallic", (22, 29, 50)),
    (150, "Light grey metallic", (171, 173, 172)),
    (151, "Sand green", (120, 144, 130)),
    (153, "Sand red", (149, 121, 119)),
    (154, "Dark red", (123, 46, 47)),
    (157, "Tr. Flu. Yellow", (255, 246, 123)),
    (158, "Tr. Flu. Red", (225, 164, 194)),
    (168, "Gun metallic", (117, 108, 98)),
    (176, "Red flip/flop", (151, 105, 91)),
    (178, "Yellow flip/flop", (180, 132, 85)),
    (179, "Silver flip/flop", (137, 135, 136)),
    (180, "Curry", (215, 169, 75)),
    (190, "Fire Yellow", (249, 214, 46)),
    (191, "Flame yellowish orange", (232, 171, 45)),
    (192, "Reddish brown", (105, 64, 40)),
    (193, "Flame reddish orange", (207, 96, 36)),
    (194, "Medium stone grey", (163, 162, 165)),
    (195, "Royal blue", (70, 103, 164)),
    (196, "Dark Royal blue", (35, 71, 139)),
    (198, "Bright reddish lilac", (142, 66, 133)),
    (199, "Dark stone grey", (99, 95, 98)),
    (200, "Lemon metalic", (130, 138, 93)),
    (208, "Light stone grey", (229, 228, 223)),
    (209, "Dark Curry", (176, 142, 68)),
    (210, "Faded green", (112, 149, 120)),
    (211, "Turquoise", (121, 181, 181)),
    (212, "Light Royal blue", (159, 195, 233)),
    (213, "Medium Royal blue", (108, 129, 183)),
    (216, "Rust", (144, 76, 42)),
    (217, "Brown", (124, 92, 70)),
    (218, "Reddish lilac", (150, 112, 159)),
    (219, "Lilac", (107, 98, 155)),
    (220, "Light lilac", (167, 169, 206)),
    (221, "Bright purple", (205, 98, 152)),
    (222, "Light purple", (228, 173, 200)),
    (223, "Light pink", (220, 144, 149)),
    (224, "Light brick yellow", (240, 213, 160)),
    (225, "Warm yellowish orange", (235, 184, 127)),
    (226, "Cool yellow", (253, 234, 141)),
    (232, "Dove blue", (125, 187, 221)),
    (268, "Medium lilac", (52, 43, 117)),
    (301, "Slime green", (80, 109, 84)),
    (302, "Smoky grey", (91, 93, 105)),
    (303, "Dark blue", (0, 16, 176)),
    (304, "Parsley green", (44, 101, 29)),
    (305, "Steel blue", (82, 124, 174)),
    (306, "Storm blue", (51, 88, 130)),
    (307, "Lapis", (16, 42, 220)),
    (308, "Dark indigo", (61, 21, 133)),
    (309, "Sea green", (52, 142, 64)),
    (310, "Shamrock", (91, 154, 76)),
    (311, "Fossil", (159, 161, 172)),
    (312, "Mulberry", (89, 34, 89)),
    (313, "Forest green", (31, 128, 29)),
    (314, "Cadet blue", (159, 173, 192)),
    (315, "Electric blue", (9, 137, 207)),
    (316, "Eggplant", (123, 0, 123)),
    (317, "Moss", (124, 156, 107)),
    (318, "Artichoke", (138, 171, 133)),
    (319, "Sage green", (185, 196, 177)),
    (320, "Ghost grey", (202, 203, 209)),
    (321, "Lilac", (167, 94, 155)),
    (322, "Plum", (123, 47, 123)),
    (323, "Olivine", (148, 190, 129)),
    (324, "Laurel green", (168, 189, 153)),
    (325, "Quill grey", (223, 223, 222)),
    (327, "Crimson", (151, 0, 0)),
    (328, "Mint", (177, 229, 166)),
    (329, "Baby blue", (152, 194, 219)),
    (330, "Carnation pink", (255, 152, 220)),
    (331, "Persimmon", (255, 89, 89)),
    (332, "Maroon", (117, 0, 0)),
    (333, "Gold", (239, 184, 56)),
    (334, "Daisy orange", (248, 217, 109)),
    (335, "Pearl", (231, 231, 236)),
    (336, "Fog", (199, 212, 228)),
    (337, "Salmon", (255, 148, 148)),
    (338, "Terra Cotta", (190, 104, 98)),
    (339, "Cocoa", (86, 36, 36)),
    (340, "Wheat", (241, 231, 199)),
    (341, "Buttermilk", (254, 243, 187)),
    (342, "Mauve", (224, 178, 208)),
    (343, "Sunrise", (212, 144, 189)),
    (344, "Tawny", (150, 85, 85)),
    (345, "Rust", (143, 76, 42)),
    (346, "Cashmere", (211, 190, 150)),
    (347, "Khaki", (226, 220, 188)),
    (348, "Lily white", (237, 234, 234)),
    (349, "Seashell", (233, 218, 218)),
    (350, "Burgundy", (136, 62, 62)),
    (351, "Cork", (188, 155, 93)),
    (352, "Burlap", (199, 172, 120)),
    (353, "Beige", (202, 191, 163)),
    (354, "Oyster", (187, 179, 178)),
    (355, "Pine Cone", (108, 88, 75)),
    (356, "Fawn brown", (160, 132, 79)),
    (357, "Hurricane grey", (149, 137, 136)),
    (358, "Cloudy grey", (171, 168, 158)),
    (359, "Linen", (175, 148, 131)),
    (360, "Copper", (150, 103, 102)),
    (361, "Dirt brown", (86, 66, 54)),
    (362, "Bronze", (126, 104, 63)),
    (363, "Flint", (105, 102, 92)),
    (364, "Dark taupe", (90, 76, 66)),
    (365, "Burnt Sienna", (106, 57, 9)),
    (1001, "Institutional white", (248, 248, 248)),
    (1002, "Mid gray", (205, 205, 205)),
    (1003, "Really black", (17, 17, 17)),
    (1004, "Really red", (255, 0, 0)),
    (1005, "Deep orange", (255, 176, 0)),
    (1006, "Alder", (180, 128, 255)),
    (1007, "Dusty Rose", (163, 75, 75)),
    (1008, "Olive", (193, 190, 66)),
    (1009, "New Yeller", (255, 255, 0)),
    (1010, "Really blue", (0, 0, 255)),
    (1011, "Navy blue", (0, 32, 96)),
    (1012, "Deep blue", (33, 84, 185)),
    (1013, "Cyan", (4, 175, 236)),
    (1014, "CGA brown", (170, 85, 0)),
    (1015, "Magenta", (170, 0, 170)),
    (1016, "Pink", (255, 102, 204)),
    (1017, "Deep orange", (255, 175, 0)),
    (1018, "Teal", (18, 238, 212)),
    (1019, "Toothpaste", (0, 255, 255)),
    (1020, "Lime green", (0, 255, 0)),
    (1021, "Camo", (58, 125, 21)),
    (1022, "Grime", (127, 142, 100)),
    (1023, "Lavender", (140, 91, 159)),
    (1024, "Pastel light blue", (175, 221, 255)),
    (1025, "Pastel orange", (255, 201, 201)),
    (1026, "Pastel violet", (177, 167, 255)),
    (1027, "Pastel blue-green", (159, 243, 233)),
    (1028, "Pastel green", (204, 255, 204)),
    (1029, "Pastel yellow", (255, 255, 204)),
    (1030, "Pastel brown", (255, 204, 153)),
    (1031, "Royal purple", (98, 37, 209)),
    (1032, "Hot pink", (255, 0, 191)),
];

const BRICK_COLOR_PALETTE: &[u16] = &[
    141, 301, 107, 26, 1012, 303, 1011, 304, 28, 1018, 302, 305, 306, 307, 308, 1021, 309, 310,
    1019, 135, 102, 23, 1010, 312, 313, 37, 1022, 1020, 1027, 311, 315, 1023, 1031, 316, 151, 317,
    318, 319, 1024, 314, 1013, 1006, 321, 322, 104, 1008, 119, 323, 324, 325, 320, 11, 1026, 1016,
    1032, 1015, 327, 1005, 1009, 29, 328, 1028, 208, 45, 329, 330, 331, 1004, 21, 332, 333, 24,
    334, 226, 1029, 335, 336, 342, 343, 338, 1007, 339, 133, 106, 340, 341, 1001, 1, 9, 1025, 337,
    344, 345, 1014, 105, 346, 347, 348, 349, 1030, 125, 101, 350, 192, 351, 352, 353, 354, 1002, 5,
    18, 217, 355, 356, 153, 357, 358, 359, 360, 38, 361, 362, 199, 194, 363, 364, 365, 1003,
];

const BRICK_COLOR_CONSTRUCTORS: &[(&str, u16)] = &[
    ("Yellow", 24),
    ("White", 1),
    ("Black", 26),
    ("Green", 28),
    ("Red", 21),
    ("DarkGray", 199),
    ("Blue", 23),
    ("Gray", 194),
];
