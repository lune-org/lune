use mlua::prelude::*;

use crate::shared::classes::add_class_restricted_method;

use super::Instance;

pub const CLASS_NAME: &str = "CollectionService";

pub fn add_methods<'lua, M: LuaUserDataMethods<'lua, Instance>>(m: &mut M) {
    add_class_restricted_method(m, CLASS_NAME, "AddTag", collection_service_add_tag);
    add_class_restricted_method(m, CLASS_NAME, "GetTags", collection_service_get_tags);
    add_class_restricted_method(m, CLASS_NAME, "HasTag", collection_service_has_tag);
    add_class_restricted_method(m, CLASS_NAME, "RemoveTag", collection_service_remove_tag);
}

/**
    Adds a tag to the instance.

    ### See Also
    * [`AddTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#AddTag)
    on the Roblox Developer Hub
*/
fn collection_service_add_tag(
    _: &Lua,
    _: &Instance,
    (object, tag_name): (Instance, String),
) -> LuaResult<()> {
    object.add_tag(tag_name);
    Ok(())
}

/**
    Gets all current tags for the instance.

    ### See Also
    * [`GetTags`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#GetTags)
    on the Roblox Developer Hub
*/
fn collection_service_get_tags(_: &Lua, _: &Instance, object: Instance) -> LuaResult<Vec<String>> {
    Ok(object.get_tags())
}

/**
    Checks if the instance has a specific tag.

    ### See Also
    * [`HasTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#HasTag)
    on the Roblox Developer Hub
*/
fn collection_service_has_tag(
    _: &Lua,
    _: &Instance,
    (object, tag_name): (Instance, String),
) -> LuaResult<bool> {
    Ok(object.has_tag(tag_name))
}

/**
    Removes a tag from the instance.

    ### See Also
    * [`RemoveTag`](https://create.roblox.com/docs/reference/engine/classes/CollectionService#RemoveTag)
    on the Roblox Developer Hub
*/
fn collection_service_remove_tag(
    _: &Lua,
    _: &Instance,
    (object, tag_name): (Instance, String),
) -> LuaResult<()> {
    object.remove_tag(tag_name);
    Ok(())
}
