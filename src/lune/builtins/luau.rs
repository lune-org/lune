use mlua::prelude::*;
use mlua::Compiler as LuaCompiler;

use crate::lune::lua::table::TableBuilder;

const DEFAULT_DEBUG_NAME: &str = "luau.load(...)";
const BYTECODE_ERROR_BYTE: u8 = 0;

struct CompileOptions {
    pub optimization_level: u8,
    pub coverage_level: u8,
    pub debug_level: u8,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            optimization_level: 1,
            coverage_level: 0,
            debug_level: 1,
        }
    }
}

impl<'lua> FromLua<'lua> for CompileOptions {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        Ok(match value {
            LuaValue::Nil => Self {
                optimization_level: 1,
                coverage_level: 0,
                debug_level: 1,
            },
            LuaValue::Table(t) => {
                let optimization_level: Option<u8> = t.get("optimizationLevel")?;
                let coverage_level: Option<u8> = t.get("coverageLevel")?;
                let debug_level: Option<u8> = t.get("debugLevel")?;

                Self {
                    optimization_level: optimization_level.unwrap_or(1),
                    coverage_level: coverage_level.unwrap_or(0),
                    debug_level: debug_level.unwrap_or(1),
                }
            }
            _ => {
                return Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "CompileOptions",
                    message: Some(format!(
                        "Invalid compile options - expected table, got {}",
                        value.type_name()
                    )),
                })
            }
        })
    }
}

struct LoadOptions {
    pub debug_name: String,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            debug_name: DEFAULT_DEBUG_NAME.to_string(),
        }
    }
}

impl<'lua> FromLua<'lua> for LoadOptions {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        Ok(match value {
            LuaValue::Nil => Self {
                debug_name: DEFAULT_DEBUG_NAME.to_string(),
            },
            LuaValue::Table(t) => {
                let debug_name: Option<String> = t.get("debugName")?;

                Self {
                    debug_name: debug_name.unwrap_or(DEFAULT_DEBUG_NAME.to_string()),
                }
            }
            LuaValue::String(s) => Self {
                debug_name: s.to_str()?.to_string(),
            },
            _ => {
                return Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "LoadOptions",
                    message: Some(format!(
                        "Invalid load options - expected string or table, got {}",
                        value.type_name()
                    )),
                })
            }
        })
    }
}

pub fn create(lua: &'static Lua) -> LuaResult<LuaTable> {
    TableBuilder::new(lua)?
        .with_function("compile", compile_source)?
        .with_function("load", load_source)?
        .build_readonly()
}

fn compile_source<'lua>(
    lua: &'lua Lua,
    (source, options): (LuaString<'lua>, CompileOptions),
) -> LuaResult<LuaString<'lua>> {
    let source_bytecode_bytes = LuaCompiler::default()
        .set_optimization_level(options.optimization_level)
        .set_coverage_level(options.coverage_level)
        .set_debug_level(options.debug_level)
        .compile(source);

    let first_byte = source_bytecode_bytes.first().unwrap();

    match *first_byte {
        BYTECODE_ERROR_BYTE => Err(LuaError::RuntimeError(
            String::from_utf8(source_bytecode_bytes).unwrap(),
        )),
        _ => lua.create_string(source_bytecode_bytes),
    }
}

fn load_source<'a>(
    lua: &'static Lua,
    (source, options): (LuaString<'a>, LoadOptions),
) -> LuaResult<LuaFunction<'a>> {
    lua.load(source.to_str()?.trim_start())
        .set_name(options.debug_name)
        .into_function()
}
