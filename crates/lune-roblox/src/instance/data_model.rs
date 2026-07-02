use mlua::prelude::*;

use crate::shared::{
    classes::{
        add_class_restricted_getter, add_class_restricted_method,
        get_or_create_property_ref_instance,
    },
    instance::class_is_a_service,
};

use super::{Instance, instance_to_lua, opt_instance_to_lua};

pub const CLASS_NAME: &str = "DataModel";

pub fn add_fields<F: LuaUserDataFields<Instance>>(f: &mut F) {
    add_class_restricted_getter(f, CLASS_NAME, "Workspace", data_model_get_workspace);
}

pub fn add_methods<M: LuaUserDataMethods<Instance>>(m: &mut M) {
    add_class_restricted_method(m, CLASS_NAME, "GetService", data_model_get_service);
    add_class_restricted_method(m, CLASS_NAME, "FindService", data_model_find_service);
}

/**
    Get the workspace parented under this datamodel, or create it if it doesn't exist.

    ### See Also
    * [`Terrain`](https://create.roblox.com/docs/reference/engine/classes/Workspace#Terrain)
      on the Roblox Developer Hub
*/
fn data_model_get_workspace(lua: &Lua, this: &Instance) -> LuaResult<LuaValue> {
    get_or_create_property_ref_instance(lua, this, "Workspace", "Workspace")
}

/**
    Gets or creates a service for this `DataModel`.

    ### See Also
    * [`GetService`](https://create.roblox.com/docs/reference/engine/classes/ServiceProvider#GetService)
      on the Roblox Developer Hub
*/
fn data_model_get_service(lua: &Lua, this: &Instance, service_name: String) -> LuaResult<LuaValue> {
    if matches!(class_is_a_service(&service_name), None | Some(false)) {
        Err(LuaError::RuntimeError(format!(
            "'{service_name}' is not a valid service name",
        )))
    } else if let Some(service) = this.find_child(|child| child.class == service_name) {
        instance_to_lua(lua, service)
    } else {
        // Create the service inside this DataModel's dom so parenting it
        // below stays within the dom (no cross-dom transfer).
        let service = Instance::new_in_dom(this.dom_id, service_name);
        service.set_parent(Some(*this));
        instance_to_lua(lua, service)
    }
}

/**
    Gets a service for this `DataModel`, if it exists.

    ### See Also
    * [`FindService`](https://create.roblox.com/docs/reference/engine/classes/ServiceProvider#FindService)
      on the Roblox Developer Hub
*/
fn data_model_find_service(
    lua: &Lua,
    this: &Instance,
    service_name: String,
) -> LuaResult<LuaValue> {
    if matches!(class_is_a_service(&service_name), None | Some(false)) {
        Err(LuaError::RuntimeError(format!(
            "'{service_name}' is not a valid service name",
        )))
    } else {
        opt_instance_to_lua(lua, this.find_child(|child| child.class == service_name))
    }
}
