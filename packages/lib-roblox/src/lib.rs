use mlua::prelude::*;

use crate::instance::Instance;

pub mod datatypes;
pub mod document;
pub mod instance;

pub(crate) mod shared;

fn make<F>(lua: &Lua, f: F) -> LuaResult<LuaValue>
where
    F: Fn(&Lua, &LuaTable) -> LuaResult<()>,
{
    let tab = lua.create_table()?;
    f(lua, &tab)?;
    tab.set_readonly(true);
    Ok(LuaValue::Table(tab))
}

#[rustfmt::skip]
fn make_all_datatypes(lua: &Lua) -> LuaResult<Vec<(&'static str, LuaValue)>> {
	use datatypes::types::*;
    Ok(vec![
		// Datatypes
        ("Axes",                   make(lua, Axes::make_table)?),
        ("BrickColor",             make(lua, BrickColor::make_table)?),
        ("CFrame",                 make(lua, CFrame::make_table)?),
        ("Color3",                 make(lua, Color3::make_table)?),
        ("ColorSequence",          make(lua, ColorSequence::make_table)?),
        ("ColorSequenceKeypoint",  make(lua, ColorSequenceKeypoint::make_table)?),
        ("Faces",                  make(lua, Faces::make_table)?),
        ("Font",                   make(lua, Font::make_table)?),
        ("NumberRange",            make(lua, NumberRange::make_table)?),
        ("NumberSequence",         make(lua, NumberSequence::make_table)?),
        ("NumberSequenceKeypoint", make(lua, NumberSequenceKeypoint::make_table)?),
        ("PhysicalProperties",     make(lua, PhysicalProperties::make_table)?),
        ("Ray",                    make(lua, Ray::make_table)?),
        ("Rect",                   make(lua, Rect::make_table)?),
        ("UDim",                   make(lua, UDim::make_table)?),
        ("UDim2",                  make(lua, UDim2::make_table)?),
        ("Region3",                make(lua, Region3::make_table)?),
        ("Region3int16",           make(lua, Region3int16::make_table)?),
        ("Vector2",                make(lua, Vector2::make_table)?),
        ("Vector2int16",           make(lua, Vector2int16::make_table)?),
        ("Vector3",                make(lua, Vector3::make_table)?),
        ("Vector3int16",           make(lua, Vector3int16::make_table)?),
		// Classes
        ("Instance", make(lua, Instance::make_table)?),
		// Singletons
        ("Enum", Enums.to_lua(lua)?),
    ])
}

pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    for (name, tab) in make_all_datatypes(lua)? {
        exports.set(name, tab)?;
    }
    exports.set_readonly(true);
    Ok(exports)
}
