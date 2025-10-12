use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::CustomPhysicalProperties as DomCustomPhysicalProperties;

use lune_utils::TableBuilder;

use crate::exports::LuaExportsTable;

use super::{super::*, EnumItem};

/**
    An implementation of the [PhysicalProperties](https://create.roblox.com/docs/reference/engine/datatypes/PhysicalProperties) Roblox datatype.

    This implements all documented properties, methods & constructors of the `PhysicalProperties` class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalProperties {
    pub(crate) density: f32,
    pub(crate) friction: f32,
    pub(crate) friction_weight: f32,
    pub(crate) elasticity: f32,
    pub(crate) elasticity_weight: f32,
    pub(crate) acoustic_absorption: f32,
}

impl PhysicalProperties {
    pub(crate) fn from_material(material_enum_item: &EnumItem) -> Option<PhysicalProperties> {
        MATERIAL_ENUM_MAP
            .iter()
            .find(|props| props.0 == material_enum_item.name)
            .map(|props| PhysicalProperties {
                density: props.1,
                friction: props.2,
                elasticity: props.3,
                friction_weight: props.4,
                elasticity_weight: props.5,
                acoustic_absorption: props.6,
            })
    }
}

impl LuaExportsTable for PhysicalProperties {
    const EXPORT_NAME: &'static str = "PhysicalProperties";

    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        type ArgsMaterial = LuaUserDataRef<EnumItem>;
        type ArgsNumbers = (f32, f32, f32, Option<f32>, Option<f32>, Option<f32>);

        let physical_properties_new = |lua: &Lua, args: LuaMultiValue| {
            if let Ok(value) = ArgsMaterial::from_lua_multi(args.clone(), lua) {
                if value.parent.desc.name == "Material" {
                    match PhysicalProperties::from_material(&value) {
                        Some(props) => Ok(props),
                        None => Err(LuaError::RuntimeError(format!(
                            "Found unknown Material '{}'",
                            value.name
                        ))),
                    }
                } else {
                    Err(LuaError::RuntimeError(format!(
                        "Expected argument #1 to be a Material, got {}",
                        value.parent.desc.name
                    )))
                }
            } else if let Ok((
                density,
                friction,
                elasticity,
                friction_weight,
                elasticity_weight,
                acoustic_absorption,
            )) = ArgsNumbers::from_lua_multi(args, lua)
            {
                Ok(PhysicalProperties {
                    density,
                    friction,
                    friction_weight: friction_weight.unwrap_or(1.0),
                    elasticity,
                    elasticity_weight: elasticity_weight.unwrap_or(1.0),
                    acoustic_absorption: acoustic_absorption.unwrap_or(1.0),
                })
            } else {
                // FUTURE: Better error message here using given arg types
                Err(LuaError::RuntimeError(
                    "Invalid arguments to constructor".to_string(),
                ))
            }
        };

        TableBuilder::new(lua)?
            .with_function("new", physical_properties_new)?
            .build_readonly()
    }
}

impl LuaUserData for PhysicalProperties {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("Density", |_, this| Ok(this.density));
        fields.add_field_method_get("Friction", |_, this| Ok(this.friction));
        fields.add_field_method_get("FrictionWeight", |_, this| Ok(this.friction_weight));
        fields.add_field_method_get("Elasticity", |_, this| Ok(this.elasticity));
        fields.add_field_method_get("ElasticityWeight", |_, this| Ok(this.elasticity_weight));
        fields.add_field_method_get("AcousticAbsorption", |_, this| Ok(this.acoustic_absorption));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for PhysicalProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {}",
            self.density,
            self.friction,
            self.elasticity,
            self.friction_weight,
            self.elasticity_weight
        )
    }
}

impl From<DomCustomPhysicalProperties> for PhysicalProperties {
    fn from(v: DomCustomPhysicalProperties) -> Self {
        Self {
            density: v.density(),
            friction: v.friction(),
            friction_weight: v.friction_weight(),
            elasticity: v.elasticity(),
            elasticity_weight: v.elasticity_weight(),
            acoustic_absorption: v.acoustic_absorption(),
        }
    }
}

impl From<PhysicalProperties> for DomCustomPhysicalProperties {
    fn from(v: PhysicalProperties) -> Self {
        DomCustomPhysicalProperties::new(
            v.density,
            v.friction,
            v.elasticity,
            v.friction_weight,
            v.elasticity_weight,
            v.acoustic_absorption,
        )
    }
}

/*

    NOTE: The material definitions below are generated using the
    physical_properties_enum_map script in the scripts dir next
    to src, which can be ran in the Roblox Studio command bar

*/
#[rustfmt::skip]
const MATERIAL_ENUM_MAP: &[(&str, f32, f32, f32, f32, f32, f32)] = &[
    ("Plastic",       0.70, 0.30, 0.50, 1.00, 1.00, 0.30),
    ("SmoothPlastic", 0.70, 0.20, 0.50, 1.00, 1.00, 0.20),
    ("Neon",          0.70, 0.30, 0.20, 1.00, 1.00, 0.30),
    ("Wood",          0.35, 0.48, 0.20, 1.00, 1.00, 0.25),
    ("WoodPlanks",    0.35, 0.48, 0.20, 1.00, 1.00, 0.25),
    ("Marble",        2.56, 0.20, 0.17, 1.00, 1.00, 0.01),
    ("Slate",         2.69, 0.40, 0.20, 1.00, 1.00, 0.03),
    ("Concrete",      2.40, 0.70, 0.20, 0.30, 1.00, 0.05),
    ("Granite",       2.69, 0.40, 0.20, 1.00, 1.00, 0.20),
    ("Brick",         1.92, 0.80, 0.15, 0.30, 1.00, 0.35),
    ("Pebble",        2.40, 0.40, 0.17, 1.00, 1.50, 0.40),
    ("Cobblestone",   2.69, 0.50, 0.17, 1.00, 1.00, 0.45),
    ("Rock",          2.69, 0.50, 0.17, 1.00, 1.00, 0.15),
    ("Sandstone",     2.69, 0.50, 0.15, 5.00, 1.00, 0.20),
    ("Basalt",        2.69, 0.70, 0.15, 0.30, 1.00, 0.20),
    ("CrackedLava",   2.69, 0.65, 0.15, 1.00, 1.00, 0.35),
    ("Limestone",     2.69, 0.50, 0.15, 1.00, 1.00, 0.15),
    ("Pavement",      2.69, 0.50, 0.17, 0.30, 1.00, 0.60),
    ("CorrodedMetal", 7.85, 0.70, 0.20, 1.00, 1.00, 0.15),
    ("DiamondPlate",  7.85, 0.35, 0.25, 1.00, 1.00, 0.05),
    ("Foil",          2.70, 0.40, 0.25, 1.00, 1.00, 0.10),
    ("Metal",         7.85, 0.40, 0.25, 1.00, 1.00, 0.10),
    ("Grass",         0.90, 0.40, 0.10, 1.00, 1.50, 0.70),
    ("LeafyGrass",    0.90, 0.40, 0.10, 2.00, 2.00, 0.75),
    ("Sand",          1.60, 0.50, 0.05, 5.00, 2.50, 0.55),
    ("Fabric",        0.70, 0.35, 0.05, 1.00, 1.00, 0.65),
    ("Snow",          0.90, 0.30, 0.03, 3.00, 4.00, 0.80),
    ("Mud",           0.90, 0.30, 0.07, 3.00, 4.00, 0.65),
    ("Ground",        0.90, 0.45, 0.10, 1.00, 1.00, 0.60),
    ("Asphalt",       2.36, 0.80, 0.20, 0.30, 1.00, 0.40),
    ("Salt",          2.16, 0.50, 0.05, 1.00, 1.00, 0.15),
    ("Ice",           0.92, 0.02, 0.15, 3.00, 1.00, 0.20),
    ("Glacier",       0.92, 0.05, 0.15, 2.00, 1.00, 0.70),
    ("Glass",         2.40, 0.25, 0.20, 1.00, 1.00, 0.10),
    ("ForceField",    2.40, 0.25, 0.20, 1.00, 1.00, 0.00),
    ("Air",           0.01, 0.01, 0.01, 1.00, 1.00, 0.01),
    ("Water",         1.00, 0.00, 0.01, 1.00, 1.00, 0.01),
    ("Cardboard",     0.70, 0.50, 0.05, 1.00, 2.00, 0.55),
    ("Carpet",        1.10, 0.40, 0.25, 1.00, 2.00, 0.65),
    ("CeramicTiles",  2.40, 0.51, 0.20, 1.00, 1.00, 0.04),
    ("ClayRoofTiles", 2.00, 0.51, 0.20, 1.00, 1.00, 0.30),
    ("RoofShingles",  2.36, 0.80, 0.20, 0.30, 1.00, 0.30),
    ("Leather",       0.86, 0.35, 0.25, 1.00, 1.00, 0.65),
    ("Plaster",       0.75, 0.60, 0.20, 0.30, 1.00, 0.30),
    ("Rubber",        1.30, 1.50, 0.95, 3.00, 2.00, 0.50),
];
