use std::str::FromStr;

use mlua::prelude::*;

use crate::get_unsafe_library_enabled;

/**
    A standard library probloxrovided by Lune.
*/
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[rustfmt::skip]
pub enum LuneStandardLibrary {
    #[cfg(feature = "datetime")] DateTime,
    #[cfg(feature = "fs")]       Fs,
    #[cfg(feature = "luau")]     Luau,
    #[cfg(feature = "net")]      Net,
    #[cfg(feature = "task")]     Task,
    #[cfg(feature = "process")]  Process,
    #[cfg(feature = "regex")]    Regex,
    #[cfg(feature = "serde")]    Serde,
    #[cfg(feature = "stdio")]    Stdio,
    #[cfg(feature = "roblox")]   Roblox,
    #[cfg(feature = "ffi")]      Ffi,
}

impl LuneStandardLibrary {
    /**
        All available standard libraries.
    */
    #[rustfmt::skip]
    pub const ALL: &'static [Self] = &[
        #[cfg(feature = "datetime")] Self::DateTime,
        #[cfg(feature = "fs")]       Self::Fs,
        #[cfg(feature = "luau")]     Self::Luau,
        #[cfg(feature = "net")]      Self::Net,
        #[cfg(feature = "task")]     Self::Task,
        #[cfg(feature = "process")]  Self::Process,
        #[cfg(feature = "regex")]    Self::Regex,
        #[cfg(feature = "serde")]    Self::Serde,
        #[cfg(feature = "stdio")]    Self::Stdio,
        #[cfg(feature = "roblox")]   Self::Roblox,
        #[cfg(feature = "ffi")]      Self::Ffi,
    ];

    /**
        Gets the name of the library, such as `datetime` or `fs`.
    */
    #[must_use]
    #[rustfmt::skip]
    #[allow(unreachable_patterns)]
    pub fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "datetime")] Self::DateTime => "datetime",
            #[cfg(feature = "fs")]       Self::Fs       => "fs",
            #[cfg(feature = "luau")]     Self::Luau     => "luau",
            #[cfg(feature = "net")]      Self::Net      => "net",
            #[cfg(feature = "task")]     Self::Task     => "task",
            #[cfg(feature = "process")]  Self::Process  => "process",
            #[cfg(feature = "regex")]    Self::Regex    => "regex",
            #[cfg(feature = "serde")]    Self::Serde    => "serde",
            #[cfg(feature = "stdio")]    Self::Stdio    => "stdio",
            #[cfg(feature = "roblox")]   Self::Roblox   => "roblox",
            #[cfg(feature = "ffi")]      Self::Ffi      => "ffi",

            _ => unreachable!("no standard library enabled"),
        }
    }

    /**
        Gets whether the library is unsafe.
    */
    #[must_use]
    #[rustfmt::skip]
    #[allow(unreachable_patterns)]
    pub fn is_unsafe(&self) -> bool {
        match self {
            #[cfg(feature = "datetime")] Self::DateTime => false,
            #[cfg(feature = "fs")]       Self::Fs       => false,
            #[cfg(feature = "luau")]     Self::Luau     => false,
            #[cfg(feature = "net")]      Self::Net      => false,
            #[cfg(feature = "task")]     Self::Task     => false,
            #[cfg(feature = "process")]  Self::Process  => false,
            #[cfg(feature = "regex")]    Self::Regex    => false,
            #[cfg(feature = "serde")]    Self::Serde    => false,
            #[cfg(feature = "stdio")]    Self::Stdio    => false,
            #[cfg(feature = "roblox")]   Self::Roblox   => false,
            #[cfg(feature = "ffi")]      Self::Ffi      => true,

            _ => unreachable!("no standard library enabled"),
        }
    }

    /**
        Creates the Lua module for the library.

        # Errors

        If the library could not be created, or if requiring an unsafe library without enabling the unsafe library.
    */
    #[rustfmt::skip]
    #[allow(unreachable_patterns)]
    pub fn module<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaMultiValue<'lua>> {
        if self.is_unsafe() && !get_unsafe_library_enabled(lua) {
            return Err(LuaError::external(format!("Standard library '{}' requires unsafe library enabled", self.name())));
        }

        let res: LuaResult<LuaTable> = match self {
            #[cfg(feature = "datetime")] Self::DateTime => lune_std_datetime::module(lua),
            #[cfg(feature = "fs")]       Self::Fs       => lune_std_fs::module(lua),
            #[cfg(feature = "luau")]     Self::Luau     => lune_std_luau::module(lua),
            #[cfg(feature = "net")]      Self::Net      => lune_std_net::module(lua),
            #[cfg(feature = "task")]     Self::Task     => lune_std_task::module(lua),
            #[cfg(feature = "process")]  Self::Process  => lune_std_process::module(lua),
            #[cfg(feature = "regex")]    Self::Regex    => lune_std_regex::module(lua),
            #[cfg(feature = "serde")]    Self::Serde    => lune_std_serde::module(lua),
            #[cfg(feature = "stdio")]    Self::Stdio    => lune_std_stdio::module(lua),
            #[cfg(feature = "roblox")]   Self::Roblox   => lune_std_roblox::module(lua),
            #[cfg(feature = "ffi")]      Self::Ffi      => lune_std_ffi::module(lua),

            _ => unreachable!("no standard library enabled"),
        };
        match res {
            Ok(v) => v.into_lua_multi(lua),
            Err(e) => Err(e.context(format!(
                "Failed to create standard library '{}'",
                self.name()
            ))),
        }
    }
}

impl FromStr for LuneStandardLibrary {
    type Err = String;
    #[rustfmt::skip]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let low = s.trim().to_ascii_lowercase();
        Ok(match low.as_str() {
            #[cfg(feature = "datetime")] "datetime" => Self::DateTime,
            #[cfg(feature = "fs")]       "fs"       => Self::Fs,
            #[cfg(feature = "luau")]     "luau"     => Self::Luau,
            #[cfg(feature = "net")]      "net"      => Self::Net,
            #[cfg(feature = "task")]     "task"     => Self::Task,
            #[cfg(feature = "process")]  "process"  => Self::Process,
            #[cfg(feature = "regex")]    "regex"    => Self::Regex,
            #[cfg(feature = "serde")]    "serde"    => Self::Serde,
            #[cfg(feature = "stdio")]    "stdio"    => Self::Stdio,
            #[cfg(feature = "roblox")]   "roblox"   => Self::Roblox,
            #[cfg(feature = "ffi")]      "ffi"      => Self::Ffi,

            _ => {
                return Err(format!(
                    "Unknown standard library '{low}'\nValid libraries are: {}",
                    Self::ALL
                        .iter()
                        .map(Self::name)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
        })
    }
}
