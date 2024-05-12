use std::str::FromStr;

use mlua::prelude::*;

/**
    A standard global provided by Lune.
*/
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum LuneStandardGlobal {
    GTable,
    Print,
    Require,
    Version,
    Warn,
}

impl LuneStandardGlobal {
    /**
        All available standard globals.
    */
    pub const ALL: &'static [Self] = &[
        Self::GTable,
        Self::Print,
        Self::Require,
        Self::Version,
        Self::Warn,
    ];

    /**
        Gets the name of the global, such as `_G` or `require`.
    */
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::GTable => "_G",
            Self::Print => "print",
            Self::Require => "require",
            Self::Version => "_VERSION",
            Self::Warn => "warn",
        }
    }

    /**
        Creates the Lua value for the global.

        # Errors

        If the global could not be created.
    */
    #[rustfmt::skip]
    #[allow(unreachable_patterns)]
    pub fn create<'lua>(&self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        let res = match self {
            Self::GTable => crate::globals::g_table::create(lua),
            Self::Print => crate::globals::print::create(lua),
            Self::Require => crate::globals::require::create(lua),
            Self::Version => crate::globals::version::create(lua),
            Self::Warn => crate::globals::warn::create(lua),
        };
        match res {
            Ok(v) => Ok(v),
            Err(e) => Err(e.context(format!(
                "Failed to create standard global '{}'",
                self.name()
            ))),
        }
    }
}

impl FromStr for LuneStandardGlobal {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let low = s.trim().to_ascii_lowercase();
        Ok(match low.as_str() {
            "_g" => Self::GTable,
            "print" => Self::Print,
            "require" => Self::Require,
            "_version" => Self::Version,
            "warn" => Self::Warn,
            _ => {
                return Err(format!(
                    "Unknown standard global '{low}'\nValid globals are: {}",
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
