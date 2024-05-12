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

fn create_all_exports(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaValue)>> {
    use datatypes::types::*;
    use instance::Instance;
    Ok(vec![
        // Datatypes
        export::<Axes>(lua)?,
        export::<BrickColor>(lua)?,
        export::<CFrame>(lua)?,
        export::<Color3>(lua)?,
        export::<ColorSequence>(lua)?,
        export::<ColorSequenceKeypoint>(lua)?,
        export::<Faces>(lua)?,
        export::<Font>(lua)?,
        export::<NumberRange>(lua)?,
        export::<NumberSequence>(lua)?,
        export::<NumberSequenceKeypoint>(lua)?,
        export::<PhysicalProperties>(lua)?,
        export::<Ray>(lua)?,
        export::<Rect>(lua)?,
        export::<UDim>(lua)?,
        export::<UDim2>(lua)?,
        export::<Region3>(lua)?,
        export::<Region3int16>(lua)?,
        export::<Vector2>(lua)?,
        export::<Vector2int16>(lua)?,
        export::<Vector3>(lua)?,
        export::<Vector3int16>(lua)?,
        // Classes
        export::<Instance>(lua)?,
        // Singletons
        ("Enum", Enums.into_lua(lua)?),
    ])
}

/**
    Creates a table containing all the Roblox datatypes, classes, and singletons.

    Note that this is not guaranteed to contain any value unless indexed directly,
    it may be optimized to use lazy initialization in the future.

    # Errors

    Errors when out of memory or when a value cannot be created.
*/
pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    // FUTURE: We can probably create these lazily as users
    // index the main exports (this return value) table and
    // save some memory and startup time. The full exports
    // table is quite big and probably won't get any smaller
    // since we impl all roblox constructors for each datatype.
    let exports = create_all_exports(lua)?;
    TableBuilder::new(lua)?
        .with_values(exports)?
        .build_readonly()
}
