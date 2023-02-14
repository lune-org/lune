use std::fmt::{Display, Formatter, Result as FmtResult};

use mlua::prelude::*;

mod fs;
mod net;
mod process;
mod require;
mod stdio;
mod task;
mod top_level;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LuneGlobal {
    Fs,
    Net,
    Process { args: Vec<String> },
    Require,
    Stdio,
    Task,
    TopLevel,
}

impl Display for LuneGlobal {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                Self::Fs => "fs",
                Self::Net => "net",
                Self::Process { .. } => "process",
                Self::Require => "require",
                Self::Stdio => "stdio",
                Self::Task => "task",
                Self::TopLevel => "toplevel",
            }
        )
    }
}

impl LuneGlobal {
    /**
        Create a vector that contains all available Lune globals, with
        the [`LuneGlobal::Process`] global containing the given args.
    */
    pub fn all<S: AsRef<str>>(args: &[S]) -> Vec<Self> {
        vec![
            Self::Fs,
            Self::Net,
            Self::Process {
                args: args.iter().map(|s| s.as_ref().to_string()).collect(),
            },
            Self::Require,
            Self::Stdio,
            Self::Task,
            Self::TopLevel,
        ]
    }

    /**
        Checks if this Lune global is a proxy global.

        A proxy global is a global that re-implements or proxies functionality of one or
        more existing lua globals, and may store internal references to the original global(s).

        This means that proxy globals should only be injected into a lua global
        environment once, since injecting twice or more will potentially break the
        functionality of the proxy global and / or cause undefined behavior.
    */
    pub fn is_proxy(&self) -> bool {
        matches!(self, Self::Require | Self::TopLevel)
    }

    /**
        Checks if this Lune global is an injector.

        An injector is similar to a proxy global but will inject
        value(s) into the global lua environment during creation,
        to ensure correct usage and compatibility with base Luau.
    */
    pub fn is_injector(&self) -> bool {
        matches!(self, Self::Task)
    }

    /**
        Creates the [`mlua::Table`] value for this Lune global.

        Note that proxy globals should be handled with special care and that [`LuneGlobal::inject()`]
        should be preferred over manually creating and manipulating the value(s) of any Lune global.
    */
    pub fn value(&self, lua: &'static Lua) -> LuaResult<LuaTable> {
        match self {
            LuneGlobal::Fs => fs::create(lua),
            LuneGlobal::Net => net::create(lua),
            LuneGlobal::Process { args } => process::create(lua, args.clone()),
            LuneGlobal::Require => require::create(lua),
            LuneGlobal::Stdio => stdio::create(lua),
            LuneGlobal::Task => task::create(lua),
            LuneGlobal::TopLevel => top_level::create(lua),
        }
    }

    /**
        Injects the Lune global into a lua global environment.

        This takes ownership since proxy Lune globals should
        only ever be injected into a lua global environment once.

        Refer to [`LuneGlobal::is_top_level()`] for more info on proxy globals.
    */
    pub fn inject(self, lua: &'static Lua) -> LuaResult<()> {
        let globals = lua.globals();
        let table = self.value(lua)?;
        // NOTE: Top level globals are special, the values
        // *in* the table they return should be set directly,
        // instead of setting the table itself as the global
        if self.is_proxy() {
            for pair in table.pairs::<LuaValue, LuaValue>() {
                let (key, value) = pair?;
                globals.raw_set(key, value)?;
            }
            Ok(())
        } else {
            globals.raw_set(self.to_string(), table)
        }
    }
}
