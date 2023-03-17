use mlua::prelude::*;

pub mod datatypes;
pub mod document;
pub mod instance;

fn make_dt<F>(lua: &Lua, f: F) -> LuaResult<LuaValue>
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
		// Classes
        ("Axes",                   make_dt(lua, Axes::make_table)?),
        ("BrickColor",             make_dt(lua, BrickColor::make_table)?),
        ("CFrame",                 make_dt(lua, CFrame::make_table)?),
        ("Color3",                 make_dt(lua, Color3::make_table)?),
        ("ColorSequence",          make_dt(lua, ColorSequence::make_table)?),
        ("ColorSequenceKeypoint",  make_dt(lua, ColorSequenceKeypoint::make_table)?),
        ("Faces",                  make_dt(lua, Faces::make_table)?),
        ("NumberRange",            make_dt(lua, NumberRange::make_table)?),
        ("NumberSequence",         make_dt(lua, NumberSequence::make_table)?),
        ("NumberSequenceKeypoint", make_dt(lua, NumberSequenceKeypoint::make_table)?),
        ("Ray",                    make_dt(lua, Ray::make_table)?),
        ("Rect",                   make_dt(lua, Rect::make_table)?),
        ("UDim",                   make_dt(lua, UDim::make_table)?),
        ("UDim2",                  make_dt(lua, UDim2::make_table)?),
        ("Region3",                make_dt(lua, Region3::make_table)?),
        ("Region3int16",           make_dt(lua, Region3int16::make_table)?),
        ("Vector2",                make_dt(lua, Vector2::make_table)?),
        ("Vector2int16",           make_dt(lua, Vector2int16::make_table)?),
        ("Vector3",                make_dt(lua, Vector3::make_table)?),
        ("Vector3int16",           make_dt(lua, Vector3int16::make_table)?),
		// Singletons
        ("Enum", LuaValue::UserData(Enums::make_singleton(lua)?)),
    ])
}

pub fn module(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;
    for (name, tab) in make_all_datatypes(lua)? {
        exports.set(name, tab)?;
    }
    Ok(exports)
}
