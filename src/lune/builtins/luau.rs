use mlua::prelude::*;
use mlua::Compiler as LuaCompiler;

use crate::lune::lua::table::TableBuilder;
// use  as LuaNumber;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("compile", compile_source)?
        .with_function("load", load_source)?
        .build_readonly()
}

fn compile_source<'lua>(
    lua: &'lua Lua,
    (source, options): (LuaString<'lua>, Option<LuaTable<'lua>>),
) -> LuaResult<LuaString<'lua>> {
    let mut optimization_level = 1;
    let mut coverage_level = 0;
    let mut debug_level = 1;

    if let Some(options) = options {
        optimization_level = match options.raw_get("optimizationLevel")? {
            LuaValue::Integer(val) => val as u8,
            _ => optimization_level,
        };

        coverage_level = match options.raw_get("coverageLevel")? {
            LuaValue::Integer(val) => val as u8,
            _ => coverage_level,
        };

        debug_level = match options.raw_get("debugLevel")? {
            LuaValue::Integer(val) => val as u8,
            _ => debug_level,
        };
    };

    let source_bytecode_bytes = LuaCompiler::default()
        .set_optimization_level(optimization_level)
        .set_coverage_level(coverage_level)
        .set_debug_level(debug_level)
        .compile(source);

    match lua.create_string(source_bytecode_bytes) {
        Ok(lua_string) => Ok(lua_string),
        Err(exception) => Err(LuaError::RuntimeError(exception.to_string())),
    }
}

fn load_source<'a>(
    lua: &'static Lua,
    (source, options): (LuaString<'a>, Option<LuaTable<'a>>),
) -> LuaResult<LuaFunction<'a>> {
    let mut lua_debug_name = None;

    if let Some(options) = options {
        lua_debug_name = match options.raw_get("debugName")? {
            LuaValue::String(val) => Some(val.to_str()?.to_string()),
            _ => lua_debug_name,
        };
    }

    let lua_object = lua
        .load(source.to_str()?.trim_start())
        .set_name(lua_debug_name.unwrap_or("luau.load(...)".to_string()))
        .into_function();

    match lua_object {
        Ok(lua_function) => Ok(lua_function),
        Err(exception) => Err(LuaError::RuntimeError(exception.to_string())),
    }
}
