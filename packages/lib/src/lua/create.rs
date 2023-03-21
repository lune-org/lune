use mlua::prelude::*;

/*
    - Level 0 is the call to info
    - Level 1 is the load call in create() below where we load this into a function
    - Level 2 is the call to the trace, which we also want to skip, so start at 3

    Also note that we must match the mlua traceback format here so that we
    can pattern match and beautify it properly later on when outputting it
*/
const TRACE_IMPL_LUA: &str = r#"
local lines = {}
for level = 3, 16 do
    local parts = {}
    local source, line, name = info(level, "sln")
    if source then
        push(parts, source)
    else
        break
    end
    if line == -1 then
        line = nil
    end
    if name and #name <= 0 then
        name = nil
    end
    if line then
        push(parts, format("%d", line))
    end
    if name and #parts > 1 then
        push(parts, format(" in function '%s'", name))
    elseif name then
        push(parts, format("in function '%s'", name))
    end
    if #parts > 0 then
        push(lines, concat(parts, ":"))
    end
end
if #lines > 0 then
    return concat(lines, "\n")
else
    return nil
end
"#;

/**
    Creates a [`mlua::Lua`] object with certain globals stored in the Lua registry.

    These globals can then be modified safely after constructing Lua using this function.

    ---
    * `"print"` -> `print`
    * `"error"` -> `error`
    ---
    * `"type"` -> `type`
    * `"typeof"` -> `typeof`
    ---
    * `"pcall"` -> `pcall`
    * `"xpcall"` -> `xpcall`
    ---
    * `"tostring"` -> `tostring`
    * `"tonumber"` -> `tonumber`
    ---
    * `"co.yield"` -> `coroutine.yield`
    * `"co.close"` -> `coroutine.close`
    ---
    * `"tab.pack"` -> `table.pack`
    * `"tab.unpack"` -> `table.unpack`
    * `"tab.freeze"` -> `table.freeze`
    * `"tab.getmeta"` -> `getmetatable`
    * `"tab.setmeta"` -> `setmetatable`
    ---
    * `"dbg.info"` -> `debug.info`
    * `"dbg.trace"` -> `debug.traceback`
    ---
*/
pub fn create() -> LuaResult<&'static Lua> {
    let lua = Lua::new().into_static();
    let globals = &lua.globals();
    let debug: LuaTable = globals.raw_get("debug")?;
    let table: LuaTable = globals.raw_get("table")?;
    let string: LuaTable = globals.raw_get("string")?;
    let coroutine: LuaTable = globals.get("coroutine")?;
    // Store original lua global functions in the registry so we can use
    // them later without passing them around and dealing with lifetimes
    lua.set_named_registry_value("print", globals.get::<_, LuaFunction>("print")?)?;
    lua.set_named_registry_value("error", globals.get::<_, LuaFunction>("error")?)?;
    lua.set_named_registry_value("type", globals.get::<_, LuaFunction>("type")?)?;
    lua.set_named_registry_value("typeof", globals.get::<_, LuaFunction>("typeof")?)?;
    lua.set_named_registry_value("xpcall", globals.get::<_, LuaFunction>("xpcall")?)?;
    lua.set_named_registry_value("pcall", globals.get::<_, LuaFunction>("pcall")?)?;
    lua.set_named_registry_value("tostring", globals.get::<_, LuaFunction>("tostring")?)?;
    lua.set_named_registry_value("tonumber", globals.get::<_, LuaFunction>("tonumber")?)?;
    lua.set_named_registry_value("co.status", coroutine.get::<_, LuaFunction>("status")?)?;
    lua.set_named_registry_value("co.yield", coroutine.get::<_, LuaFunction>("yield")?)?;
    lua.set_named_registry_value("co.close", coroutine.get::<_, LuaFunction>("close")?)?;
    lua.set_named_registry_value("dbg.info", debug.get::<_, LuaFunction>("info")?)?;
    lua.set_named_registry_value("tab.pack", table.get::<_, LuaFunction>("pack")?)?;
    lua.set_named_registry_value("tab.unpack", table.get::<_, LuaFunction>("unpack")?)?;
    lua.set_named_registry_value("tab.freeze", table.get::<_, LuaFunction>("freeze")?)?;
    lua.set_named_registry_value(
        "tab.getmeta",
        globals.get::<_, LuaFunction>("getmetatable")?,
    )?;
    lua.set_named_registry_value(
        "tab.setmeta",
        globals.get::<_, LuaFunction>("setmetatable")?,
    )?;
    // Create a trace function that can be called to obtain a full stack trace from
    // lua, this is not possible to do from rust when using our manual scheduler
    let dbg_trace_env = lua.create_table_with_capacity(0, 1)?;
    dbg_trace_env.set("info", debug.get::<_, LuaFunction>("info")?)?;
    dbg_trace_env.set("push", table.get::<_, LuaFunction>("insert")?)?;
    dbg_trace_env.set("concat", table.get::<_, LuaFunction>("concat")?)?;
    dbg_trace_env.set("format", string.get::<_, LuaFunction>("format")?)?;
    let dbg_trace_fn = lua
        .load(TRACE_IMPL_LUA)
        .set_name("=dbg.trace")?
        .set_environment(dbg_trace_env)?
        .into_function()?;
    lua.set_named_registry_value("dbg.trace", dbg_trace_fn)?;
    // All done
    Ok(lua)
}
