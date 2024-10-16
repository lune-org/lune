use mlua::prelude::*;

use crate::shared::classes::{add_class_restricted_getter, get_or_create_property_ref_instance};

use super::Instance;

pub const CLASS_NAME: &str = "Workspace";

pub fn add_fields<'lua, F: LuaUserDataFields<'lua, Instance>>(f: &mut F) {
    add_class_restricted_getter(f, CLASS_NAME, "Terrain", workspace_get_terrain);
    add_class_restricted_getter(f, CLASS_NAME, "CurrentCamera", workspace_get_camera);
}

/**
    Get the terrain parented under this workspace, or create it if it doesn't exist.

    ### See Also
    * [`Terrain`](https://create.roblox.com/docs/reference/engine/classes/Workspace#Terrain)
      on the Roblox Developer Hub
*/
fn workspace_get_terrain(_: &Lua, this: &Instance) -> LuaResult<Instance> {
    get_or_create_property_ref_instance(this, "Terrain", "Terrain")
}

/**
    Get the camera parented under this workspace, or create it if it doesn't exist.

    ### See Also
    * [`CurrentCamera`](https://create.roblox.com/docs/reference/engine/classes/Workspace#CurrentCamera)
      on the Roblox Developer Hub
*/
fn workspace_get_camera(_: &Lua, this: &Instance) -> LuaResult<Instance> {
    get_or_create_property_ref_instance(this, "CurrentCamera", "Camera")
}
