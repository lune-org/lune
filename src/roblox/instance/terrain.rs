use mlua::prelude::*;
use rbx_dom_weak::types::{MaterialColors, TerrainMaterials, Variant};

use crate::roblox::{
    datatypes::types::{Color3, EnumItem},
    shared::classes::{add_class_restricted_method, add_class_restricted_method_mut},
};

use super::Instance;

pub const CLASS_NAME: &str = "Terrain";

fn material_from_name(material_name: &str) -> Option<TerrainMaterials> {
    match material_name {
        "Grass" => Some(TerrainMaterials::Grass),
        "Slate" => Some(TerrainMaterials::Slate),
        "Concrete" => Some(TerrainMaterials::Concrete),
        "Brick" => Some(TerrainMaterials::Brick),
        "Sand" => Some(TerrainMaterials::Sand),
        "WoodPlanks" => Some(TerrainMaterials::WoodPlanks),
        "Rock" => Some(TerrainMaterials::Rock),
        "Glacier" => Some(TerrainMaterials::Glacier),
        "Snow" => Some(TerrainMaterials::Snow),
        "Sandstone" => Some(TerrainMaterials::Sandstone),
        "Mud" => Some(TerrainMaterials::Mud),
        "Basalt" => Some(TerrainMaterials::Basalt),
        "Ground" => Some(TerrainMaterials::Ground),
        "CrackedLava" => Some(TerrainMaterials::CrackedLava),
        "Asphalt" => Some(TerrainMaterials::Asphalt),
        "Cobblestone" => Some(TerrainMaterials::Cobblestone),
        "Ice" => Some(TerrainMaterials::Ice),
        "LeafyGrass" => Some(TerrainMaterials::LeafyGrass),
        "Salt" => Some(TerrainMaterials::Salt),
        "Limestone" => Some(TerrainMaterials::Limestone),
        "Pavement" => Some(TerrainMaterials::Pavement),
        _ => None,
    }
}

pub fn add_methods<'lua, M: LuaUserDataMethods<'lua, Instance>>(methods: &mut M) {
    add_class_restricted_method(
        methods,
        CLASS_NAME,
        "GetMaterialColor",
        terrain_get_material_color,
    );

    add_class_restricted_method_mut(
        methods,
        CLASS_NAME,
        "SetMaterialColor",
        terrain_set_material_color,
    )
}

fn get_or_create_material_colors(instance: &Instance) -> MaterialColors {
    if let Some(Variant::MaterialColors(material_colors)) = instance.get_property("MaterialColors")
    {
        material_colors
    } else {
        MaterialColors::default()
    }
}

/**
    Returns the color of the given terrain material.

    ### See Also
    * [`GetMaterialColor`](https://create.roblox.com/docs/reference/engine/classes/Terrain#GetMaterialColor)
    on the Roblox Developer Hub
*/
fn terrain_get_material_color(_: &Lua, this: &Instance, material: EnumItem) -> LuaResult<Color3> {
    let material_colors = get_or_create_material_colors(this);

    if &material.parent.desc.name != "Material" {
        return Err(LuaError::RuntimeError(format!(
            "Expected Material, got {}",
            &material.parent.desc.name
        )));
    }

    if let Some(terrain_material) = material_from_name(&material.name) {
        Ok(material_colors.get_color(terrain_material).into())
    } else {
        Err(LuaError::RuntimeError(format!(
            "{} is not a valid Terrain material",
            &material.name
        )))
    }
}

/**
    Sets the color of the given terrain material.

    ### See Also
    * [`SetMaterialColor`](https://create.roblox.com/docs/reference/engine/classes/Terrain#SetMaterialColor)
    on the Roblox Developer Hub
*/
fn terrain_set_material_color(
    _: &Lua,
    this: &mut Instance,
    args: (EnumItem, Color3),
) -> LuaResult<()> {
    let mut material_colors = get_or_create_material_colors(this);
    let material = args.0;
    let color = args.1;

    if &material.parent.desc.name != "Material" {
        return Err(LuaError::RuntimeError(format!(
            "Expected Material, got {}",
            &material.parent.desc.name
        )));
    }

    let terrain_material = if let Some(terrain_material) = material_from_name(&material.name) {
        terrain_material
    } else {
        return Err(LuaError::RuntimeError(format!(
            "{} is not a valid Terrain material",
            &material.name
        )));
    };

    material_colors.set_color(terrain_material, color.into());
    this.set_property("MaterialColors", Variant::MaterialColors(material_colors));
    Ok(())
}
