use mlua::prelude::*;

/*
    - Level 0 is the call to info
    - Level 1 is the load call in create() below where we load this into a function
    - Level 2 is the call to the scheduler, probably, but we can't know for sure so we start at 2
*/
const TRACE_IMPL_LUA: &str = r#"
local lines = {}
for level = 2, 2^8 do
    local source, line, name = info(level, "sln")
    if source then
        if line then
            if name and #name > 0 then
                push(lines, format("    Script '%s', Line %d - function %s", source, line, name))
            else
                push(lines, format("    Script '%s', Line %d", source, line))
            end
        elseif name and #name > 0 then
            push(lines, format("    Script '%s' - function %s", source, name))
        else
            push(lines, format("    Script '%s'", source))
        end
    elseif name then
        push(lines, format("[Lune] - function %s", source, name))
    else
        break
    end
end
if #lines > 0 then
    push(lines, 1, "Stack Begin")
    push(lines, "Stack End")
    return concat(lines, "\n")
else
    return nil
end
"#;

/**
    Creates a [`mlua::Lua`] object with certain globals stored in the Lua registry.

    These globals can then be modified safely after constructing Lua using this function.

    ---
    * `"require"` -> `require`
    ---
    * `"print"` -> `print`
    * `"error"` -> `error`
    ---
    * `"type"` -> `type`
    * `"typeof"` -> `typeof`
    ---
    * `"co.thread"` -> `coroutine.running`
    * `"co.yield"` -> `coroutine.yield`
    * `"co.close"` -> `coroutine.close`
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
    lua.set_named_registry_value("require", globals.get::<_, LuaFunction>("require")?)?;
    lua.set_named_registry_value("print", globals.get::<_, LuaFunction>("print")?)?;
    lua.set_named_registry_value("error", globals.get::<_, LuaFunction>("error")?)?;
    lua.set_named_registry_value("type", globals.get::<_, LuaFunction>("type")?)?;
    lua.set_named_registry_value("typeof", globals.get::<_, LuaFunction>("typeof")?)?;
    lua.set_named_registry_value("co.thread", coroutine.get::<_, LuaFunction>("running")?)?;
    lua.set_named_registry_value("co.yield", coroutine.get::<_, LuaFunction>("yield")?)?;
    lua.set_named_registry_value("co.close", coroutine.get::<_, LuaFunction>("close")?)?;
    lua.set_named_registry_value("dbg.info", debug.get::<_, LuaFunction>("info")?)?;
    // Create a trace function that can be called to obtain a full stack trace from
    // lua, this is not possible to do from rust when using our manual scheduler
    let trace_env = lua.create_table_with_capacity(0, 1)?;
    trace_env.set("info", debug.get::<_, LuaFunction>("info")?)?;
    trace_env.set("push", table.get::<_, LuaFunction>("insert")?)?;
    trace_env.set("concat", table.get::<_, LuaFunction>("concat")?)?;
    trace_env.set("format", string.get::<_, LuaFunction>("format")?)?;
    let trace_fn = lua
        .load(TRACE_IMPL_LUA)
        .set_environment(trace_env)?
        .into_function()?;
    lua.set_named_registry_value("dbg.trace", trace_fn)?;
    // All done
    Ok(lua)
}
