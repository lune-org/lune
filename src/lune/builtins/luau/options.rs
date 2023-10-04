use mlua::prelude::*;
use mlua::Compiler as LuaCompiler;

const DEFAULT_DEBUG_NAME: &str = "luau.load(...)";

pub struct LuauCompileOptions {
    pub(crate) optimization_level: u8,
    pub(crate) coverage_level: u8,
    pub(crate) debug_level: u8,
}

impl LuauCompileOptions {
    pub fn into_compiler(self) -> LuaCompiler {
        LuaCompiler::default()
            .set_optimization_level(self.optimization_level)
            .set_coverage_level(self.coverage_level)
            .set_debug_level(self.debug_level)
    }
}

impl Default for LuauCompileOptions {
    fn default() -> Self {
        // NOTE: This is the same as LuaCompiler::default() values, but they are
        // not accessible from outside of mlua so we need to recreate them here.
        Self {
            optimization_level: 1,
            coverage_level: 0,
            debug_level: 1,
        }
    }
}

impl<'lua> FromLua<'lua> for LuauCompileOptions {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        Ok(match value {
            LuaValue::Nil => Self::default(),
            LuaValue::Table(t) => {
                let mut options = Self::default();

                let get_and_check = |name: &'static str| -> LuaResult<Option<u8>> {
                    match t.get(name)? {
                        Some(n @ (0..=2)) => Ok(Some(n)),
                        Some(n) => Err(LuaError::runtime(format!(
                            "'{name}' must be one of: 0, 1, or 2 - got {n}"
                        ))),
                        None => Ok(None),
                    }
                };

                if let Some(optimization_level) = get_and_check("optimizationLevel")? {
                    options.optimization_level = optimization_level;
                }
                if let Some(coverage_level) = get_and_check("coverageLevel")? {
                    options.coverage_level = coverage_level;
                }
                if let Some(debug_level) = get_and_check("debugLevel")? {
                    options.debug_level = debug_level;
                }

                options
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

pub struct LuauLoadOptions<'lua> {
    pub(crate) debug_name: String,
    pub(crate) environment: Option<LuaTable<'lua>>,
}

impl Default for LuauLoadOptions<'_> {
    fn default() -> Self {
        Self {
            debug_name: DEFAULT_DEBUG_NAME.to_string(),
            environment: None,
        }
    }
}

impl<'lua> FromLua<'lua> for LuauLoadOptions<'lua> {
    fn from_lua(value: LuaValue<'lua>, _: &'lua Lua) -> LuaResult<Self> {
        Ok(match value {
            LuaValue::Nil => Self::default(),
            LuaValue::Table(t) => {
                let mut options = Self::default();

                if let Some(debug_name) = t.get("debugName")? {
                    options.debug_name = debug_name;
                }

                if let Some(environment) = t.get("environment")? {
                    options.environment = Some(environment);
                }

                options
            }
            LuaValue::String(s) => Self {
                debug_name: s.to_string_lossy().to_string(),
                environment: None,
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
