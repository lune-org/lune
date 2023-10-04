use mlua::prelude::*;

use crate::lune::util::TableBuilder;

mod options;
use options::{LuauCompileOptions, LuauLoadOptions};

const BYTECODE_ERROR_BYTE: u8 = 0;

pub fn create(lua: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("compile", compile_source)?
        .with_function("load", load_source)?
        .build_readonly()
}

fn compile_source<'lua>(
    lua: &'lua Lua,
    (source, options): (LuaString<'lua>, LuauCompileOptions),
) -> LuaResult<LuaString<'lua>> {
    let bytecode = options.into_compiler().compile(source);

    match bytecode.first() {
        Some(&BYTECODE_ERROR_BYTE) => Err(LuaError::RuntimeError(
            String::from_utf8_lossy(&bytecode).into_owned(),
        )),
        Some(_) => lua.create_string(bytecode),
        None => panic!("Compiling resulted in empty bytecode"),
    }
}

fn load_source<'lua>(
    lua: &'lua Lua,
    (source, options): (LuaString<'lua>, LuauLoadOptions),
) -> LuaResult<LuaFunction<'lua>> {
    let mut chunk = lua.load(source.as_bytes()).set_name(options.debug_name);

    if let Some(environment) = options.environment {
        let environment_with_globals = lua.create_table()?;

        if let Some(meta) = environment.get_metatable() {
            environment_with_globals.set_metatable(Some(meta));
        }

        for pair in lua.globals().pairs() {
            let (key, value): (LuaValue, LuaValue) = pair?;
            environment_with_globals.set(key, value)?;
        }

        for pair in environment.pairs() {
            let (key, value): (LuaValue, LuaValue) = pair?;
            environment_with_globals.set(key, value)?;
        }

        chunk = chunk.set_environment(environment_with_globals);
    }

    chunk.into_function()
}
