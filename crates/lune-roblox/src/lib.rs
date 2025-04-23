#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

use lune_utils::TableBuilder;

pub mod datatypes;
pub mod document;
pub mod instance;
pub mod reflection;

pub(crate) mod exports;
pub(crate) mod shared;

use exports::export;

fn create_all_exports(lua: Lua) -> LuaResult<Vec<(&'static str, LuaValue)>> {
    use datatypes::types::*;
    use instance::Instance;
    Ok(vec![
        // Datatypes
        export::<Axes>(lua.clone())?,
        export::<BrickColor>(lua.clone())?,
        export::<CFrame>(lua.clone())?,
        export::<Color3>(lua.clone())?,
        export::<ColorSequence>(lua.clone())?,
        export::<ColorSequenceKeypoint>(lua.clone())?,
        export::<Content>(lua.clone())?,
        export::<Faces>(lua.clone())?,
        export::<Font>(lua.clone())?,
        export::<NumberRange>(lua.clone())?,
        export::<NumberSequence>(lua.clone())?,
        export::<NumberSequenceKeypoint>(lua.clone())?,
        export::<PhysicalProperties>(lua.clone())?,
        export::<Ray>(lua.clone())?,
        export::<Rect>(lua.clone())?,
        export::<UDim>(lua.clone())?,
        export::<UDim2>(lua.clone())?,
        export::<Region3>(lua.clone())?,
        export::<Region3int16>(lua.clone())?,
        export::<Vector2>(lua.clone())?,
        export::<Vector2int16>(lua.clone())?,
        export::<Vector3>(lua.clone())?,
        export::<Vector3int16>(lua.clone())?,
        // Classes
        export::<Instance>(lua.clone())?,
        // Singletons
        ("Enum", Enums.into_lua(&lua)?),
    ])
}

/**
    Creates a table containing all the Roblox datatypes, classes, and singletons.

    Note that this is not guaranteed to contain any value unless indexed directly,
    it may be optimized to use lazy initialization in the future.

    # Errors

    Errors when out of memory or when a value cannot be created.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    // FUTURE: We can probably create these lazily as users
    // index the main exports (this return value) table and
    // save some memory and startup time. The full exports
    // table is quite big and probably won't get any smaller
    // since we impl all roblox constructors for each datatype.
    let exports = create_all_exports(lua.clone())?;
    TableBuilder::new(lua)?
        .with_values(exports)?
        .build_readonly()
}
