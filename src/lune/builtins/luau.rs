use mlua::prelude::*;
use mlua::Compiler as LuaCompiler;

use crate::lua::table::TableBuilder;

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("compile", compile_source)?
        .with_function("load", load_source)?
        .build_readonly()
}

fn compile_source<'a>(
    lua: &'static Lua,
    (source, options): (LuaString<'a>, Option<LuaTable<'a>>),
) -> LuaResult<LuaString<'a>> {
    let mut optimization_level = 1;
    let mut coverage_level = 0;
    let mut debug_level = 1;

    if let Some(options) = options {
        optimization_level = options.raw_get("optimizationLevel")?;
        coverage_level = options.raw_get("coverageLevel")?;
        debug_level = options.raw_get("debugLevel")?;
    }

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
    let mut lua_debug_name = "".to_string();

    if let Some(options) = options {
        lua_debug_name = options.raw_get("debugName")?
    }

    let lua_object = lua
        .load(source.to_str()?.trim_start())
        .set_name(lua_debug_name)
        .into_function();

    match lua_object {
        Ok(lua_function) => Ok(lua_function),
        Err(exception) => Err(LuaError::RuntimeError(exception.to_string())),
    }
}
