use mlua::{prelude::*, Compiler as LuaCompiler};

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
    Stores the following globals in the Lua registry:

    | Registry Name   | Global            |
    |-----------------|-------------------|
    | `"print"`       | `print`           |
    | `"error"`       | `error`           |
    | `"type"`        | `type`            |
    | `"typeof"`      | `typeof`          |
    | `"pcall"`       | `pcall`           |
    | `"xpcall"`      | `xpcall`          |
    | `"tostring"`    | `tostring`        |
    | `"tonumber"`    | `tonumber`        |
    | `"co.yield"`    | `coroutine.yield` |
    | `"co.close"`    | `coroutine.close` |
    | `"tab.pack"`    | `table.pack`      |
    | `"tab.unpack"`  | `table.unpack`    |
    | `"tab.freeze"`  | `table.freeze`    |
    | `"tab.getmeta"` | `getmetatable`    |
    | `"tab.setmeta"` | `setmetatable`    |
    | `"dbg.info"`    | `debug.info`      |
    | `"dbg.trace"`   | `debug.traceback` |

    These globals can then be modified safely from other runtime code.
*/
fn store_globals_in_registry(lua: &Lua) -> LuaResult<()> {
    // Extract some global tables that we will extract
    // built-in functions from and store in the registry
    let globals = lua.globals();
    let debug: LuaTable = globals.get("debug")?;
    let string: LuaTable = globals.get("string")?;
    let table: LuaTable = globals.get("table")?;
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
        .set_name("=dbg.trace")
        .set_environment(dbg_trace_env)
        .into_function()?;
    lua.set_named_registry_value("dbg.trace", dbg_trace_fn)?;

    Ok(())
}

/**
    Sets the `_VERSION` global to a value matching the string `Lune x.y.z+w` where
    `x.y.z` is the current Lune runtime version and `w` is the current Luau version
*/
fn set_global_version(lua: &Lua) -> LuaResult<()> {
    let luau_version_full = lua
        .globals()
        .get::<_, LuaString>("_VERSION")
        .expect("Missing _VERSION global");
    let luau_version = luau_version_full
        .to_str()?
        .strip_prefix("Luau 0.")
        .expect("_VERSION global is formatted incorrectly")
        .trim();
    if luau_version.is_empty() {
        panic!("_VERSION global is missing version number")
    }
    lua.globals().set(
        "_VERSION",
        lua.create_string(&format!(
            "Lune {lune}+{luau}",
            lune = env!("CARGO_PKG_VERSION"),
            luau = luau_version,
        ))?,
    )
}

/**
    Creates a _G table that is separate from our built-in globals
*/
fn set_global_table(lua: &Lua) -> LuaResult<()> {
    lua.globals().set("_G", lua.create_table()?)
}

/**
    Enables JIT and sets default compiler settings for the Lua struct.
*/
fn init_compiler_settings(lua: &Lua) {
    lua.enable_jit(true);
    lua.set_compiler(
        LuaCompiler::default()
            .set_coverage_level(0)
            .set_debug_level(1)
            .set_optimization_level(1),
    );
}

/**
    Creates a new [`mlua::Lua`] struct with compiler,
    registry, and globals customized for the Lune runtime.

    Refer to the source code for additional details and specifics.
*/
pub fn create() -> LuaResult<Lua> {
    let lua = Lua::new();
    init_compiler_settings(&lua);
    store_globals_in_registry(&lua)?;
    set_global_version(&lua)?;
    set_global_table(&lua)?;
    Ok(lua)
}
