use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::CustomPhysicalProperties as DomCustomPhysicalProperties;

use crate::{lune::util::TableBuilder, roblox::exports::LuaExportsTable};

use super::{super::*, EnumItem};

/**
    An implementation of the [PhysicalProperties](https://create.roblox.com/docs/reference/engine/datatypes/PhysicalProperties) Roblox datatype.

    This implements all documented properties, methods & constructors of the PhysicalProperties class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalProperties {
    pub(crate) density: f32,
    pub(crate) friction: f32,
    pub(crate) friction_weight: f32,
    pub(crate) elasticity: f32,
    pub(crate) elasticity_weight: f32,
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
            })
    }
}

impl LuaExportsTable<'_> for PhysicalProperties {
    const EXPORT_NAME: &'static str = "PhysicalProperties";

    fn create_exports_table(lua: &Lua) -> LuaResult<LuaTable> {
        type ArgsMaterial<'lua> = LuaUserDataRef<'lua, EnumItem>;
        type ArgsNumbers = (f32, f32, f32, Option<f32>, Option<f32>);

        let physical_properties_new = |lua, args: LuaMultiValue| {
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
            } else if let Ok((density, friction, elasticity, friction_weight, elasticity_weight)) =
                ArgsNumbers::from_lua_multi(args, lua)
            {
                Ok(PhysicalProperties {
                    density,
                    friction,
                    friction_weight: friction_weight.unwrap_or(1.0),
                    elasticity,
                    elasticity_weight: elasticity_weight.unwrap_or(1.0),
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
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Density", |_, this| Ok(this.density));
        fields.add_field_method_get("Friction", |_, this| Ok(this.friction));
        fields.add_field_method_get("FrictionWeight", |_, this| Ok(this.friction_weight));
        fields.add_field_method_get("Elasticity", |_, this| Ok(this.elasticity));
        fields.add_field_method_get("ElasticityWeight", |_, this| Ok(this.elasticity_weight));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
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
            density: v.density,
            friction: v.friction,
            friction_weight: v.friction_weight,
            elasticity: v.elasticity,
            elasticity_weight: v.elasticity_weight,
        }
    }
}

impl From<PhysicalProperties> for DomCustomPhysicalProperties {
    fn from(v: PhysicalProperties) -> Self {
        DomCustomPhysicalProperties {
            density: v.density,
            friction: v.friction,
            friction_weight: v.friction_weight,
            elasticity: v.elasticity,
            elasticity_weight: v.elasticity_weight,
        }
    }
}

/*

    NOTE: The material definitions below are generated using the
    physical_properties_enum_map script in the scripts dir next
    to src, which can be ran in the Roblox Studio command bar

*/

#[rustfmt::skip]
const MATERIAL_ENUM_MAP: &[(&str, f32, f32, f32, f32, f32)] = &[
    ("Plastic",       0.70, 0.30, 0.50, 1.00, 1.00),
    ("Wood",          0.35, 0.48, 0.20, 1.00, 1.00),
    ("Slate",         2.69, 0.40, 0.20, 1.00, 1.00),
    ("Concrete",      2.40, 0.70, 0.20, 0.30, 1.00),
    ("CorrodedMetal", 7.85, 0.70, 0.20, 1.00, 1.00),
    ("DiamondPlate",  7.85, 0.35, 0.25, 1.00, 1.00),
    ("Foil",          2.70, 0.40, 0.25, 1.00, 1.00),
    ("Grass",         0.90, 0.40, 0.10, 1.00, 1.50),
    ("Ice",           0.92, 0.02, 0.15, 3.00, 1.00),
    ("Marble",        2.56, 0.20, 0.17, 1.00, 1.00),
    ("Granite",       2.69, 0.40, 0.20, 1.00, 1.00),
    ("Brick",         1.92, 0.80, 0.15, 0.30, 1.00),
    ("Pebble",        2.40, 0.40, 0.17, 1.00, 1.50),
    ("Sand",          1.60, 0.50, 0.05, 5.00, 2.50),
    ("Fabric",        0.70, 0.35, 0.05, 1.00, 1.00),
    ("SmoothPlastic", 0.70, 0.20, 0.50, 1.00, 1.00),
    ("Metal",         7.85, 0.40, 0.25, 1.00, 1.00),
    ("WoodPlanks",    0.35, 0.48, 0.20, 1.00, 1.00),
    ("Cobblestone",   2.69, 0.50, 0.17, 1.00, 1.00),
    ("Air",           0.01, 0.01, 0.01, 1.00, 1.00),
    ("Water",         1.00, 0.00, 0.01, 1.00, 1.00),
    ("Rock",          2.69, 0.50, 0.17, 1.00, 1.00),
    ("Glacier",       0.92, 0.05, 0.15, 2.00, 1.00),
    ("Snow",          0.90, 0.30, 0.03, 3.00, 4.00),
    ("Sandstone",     2.69, 0.50, 0.15, 5.00, 1.00),
    ("Mud",           0.90, 0.30, 0.07, 3.00, 4.00),
    ("Basalt",        2.69, 0.70, 0.15, 0.30, 1.00),
    ("Ground",        0.90, 0.45, 0.10, 1.00, 1.00),
    ("CrackedLava",   2.69, 0.65, 0.15, 1.00, 1.00),
    ("Neon",          0.70, 0.30, 0.20, 1.00, 1.00),
    ("Glass",         2.40, 0.25, 0.20, 1.00, 1.00),
    ("Asphalt",       2.36, 0.80, 0.20, 0.30, 1.00),
    ("LeafyGrass",    0.90, 0.40, 0.10, 2.00, 2.00),
    ("Salt",          2.16, 0.50, 0.05, 1.00, 1.00),
    ("Limestone",     2.69, 0.50, 0.15, 1.00, 1.00),
    ("Pavement",      2.69, 0.50, 0.17, 0.30, 1.00),
    ("ForceField",    2.40, 0.25, 0.20, 1.00, 1.00),
];
