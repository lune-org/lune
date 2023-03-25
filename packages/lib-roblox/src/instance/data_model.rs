use mlua::prelude::*;

use crate::shared::{classes::add_class_restricted_method, instance::class_is_a_service};

use super::Instance;

pub const CLASS_NAME: &str = "DataModel";

pub fn add_methods<'lua, M: LuaUserDataMethods<'lua, Instance>>(m: &mut M) {
    add_class_restricted_method(m, CLASS_NAME, "GetService", data_model_get_service);
    add_class_restricted_method(m, CLASS_NAME, "FindService", data_model_find_service);
}

/**
    Gets or creates a service for this DataModel.

    ### See Also
    * [`GetService`](https://create.roblox.com/docs/reference/engine/classes/ServiceProvider#GetService)
    on the Roblox Developer Hub
*/
fn data_model_get_service(_: &Lua, this: &Instance, service_name: String) -> LuaResult<Instance> {
    if matches!(class_is_a_service(&service_name), None | Some(false)) {
        Err(LuaError::RuntimeError(format!(
            "'{}' is not a valid service name",
            service_name
        )))
    } else if let Some(service) = this.find_child(|child| child.class == service_name) {
        Ok(service)
    } else {
        let service = Instance::new_orphaned(service_name);
        service.set_parent(Some(this.clone()));
        Ok(service)
    }
}

/**
    Gets a service for this DataModel, if it exists.

    ### See Also
    * [`FindService`](https://create.roblox.com/docs/reference/engine/classes/ServiceProvider#FindService)
    on the Roblox Developer Hub
*/
fn data_model_find_service(
    _: &Lua,
    this: &Instance,
    service_name: String,
) -> LuaResult<Option<Instance>> {
    if matches!(class_is_a_service(&service_name), None | Some(false)) {
        Err(LuaError::RuntimeError(format!(
            "'{}' is not a valid service name",
            service_name
        )))
    } else if let Some(service) = this.find_child(|child| child.class == service_name) {
        Ok(Some(service))
    } else {
        Ok(None)
    }
}
