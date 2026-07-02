#![allow(clippy::cargo_common_metadata)]

use mlua::prelude::*;

use lune_utils::{TableBuilder, jit::JitEnablement};

mod options;

use self::options::{LuauCompileOptions, LuauLoadOptions};

const TYPEDEFS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/types.d.luau"));

/**
    Returns a string containing type definitions for the `luau` standard library.
*/
#[must_use]
pub fn typedefs() -> String {
    TYPEDEFS.to_string()
}

/**
    Creates the `luau` standard library module.

    # Errors

    Errors when out of memory.
*/
pub fn module(lua: Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("compile", compile_source)?
        .with_function("load", load_source)?
        .build_readonly()
}

fn compile_source(
    lua: &Lua,
    (source, options): (LuaString, LuauCompileOptions),
) -> LuaResult<LuaString> {
    options
        .into_compiler()
        .compile(source.as_bytes())
        .and_then(|s| lua.create_string(s))
}

fn load_source(
    lua: &Lua,
    (source, options): (LuaString, LuauLoadOptions),
) -> LuaResult<LuaFunction> {
    let mut chunk = lua
        .load(source.as_bytes().to_vec())
        .set_name(options.debug_name);
    let env_changed = options.environment.is_some();

    if let Some(custom_environment) = options.environment {
        let environment = lua.create_table()?;

        // Inject all globals into the environment
        if options.inject_globals {
            for pair in lua.globals().pairs() {
                let (key, value): (LuaValue, LuaValue) = pair?;
                environment.set(key, value)?;
            }

            if let Some(global_metatable) = lua.globals().metatable() {
                environment.set_metatable(Some(global_metatable))?;
            }
        } else if let Some(custom_metatable) = custom_environment.metatable() {
            // Since we don't need to set the global metatable,
            // we can just set a custom metatable if it exists
            environment.set_metatable(Some(custom_metatable))?;
        }

        // Inject the custom environment
        for pair in custom_environment.pairs() {
            let (key, value): (LuaValue, LuaValue) = pair?;
            environment.set(key, value)?;
        }

        chunk = chunk.set_environment(environment);
    }

    // Enable JIT if codegen is enabled and the environment hasn't
    // changed, otherwise disable JIT since it'll fall back anyways
    lua.enable_jit(options.codegen_enabled && !env_changed);
    let function = chunk.into_function()?;
    lua.enable_jit(
        lua.app_data_ref::<JitEnablement>()
            .ok_or(LuaError::runtime(
                "Failed to get current JitStatus ref from AppData",
            ))?
            .enabled(),
    );

    Ok(function)
}
