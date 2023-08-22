use std::process::ExitCode;

use mlua::{Lua, Table as LuaTable, Value as LuaValue};

mod builtins;
mod error;
mod globals;
mod scheduler;
mod util;

use self::scheduler::{LuaSchedulerExt, Scheduler};

pub use error::LuneError;

#[derive(Debug, Clone)]
pub struct Lune {
    lua: &'static Lua,
    scheduler: &'static Scheduler<'static>,
    args: Vec<String>,
}

impl Lune {
    /**
        Creates a new Lune runtime, with a new Luau VM and task scheduler.
    */
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // FIXME: Leaking these here does not feel great... is there
        // any way for us to create a scheduler, store it in app data, and
        // guarantee it has the same lifetime as Lua without using any unsafe?
        let lua = Lua::new().into_static();
        let scheduler = Scheduler::new().into_static();

        lua.set_scheduler(scheduler);
        globals::inject_all(lua).expect("Failed to inject lua globals");

        Self {
            lua,
            scheduler,
            args: Vec::new(),
        }
    }

    /**
        Sets arguments to give in `process.args` for Lune scripts.
    */
    pub fn with_args<V>(mut self, args: V) -> Self
    where
        V: Into<Vec<String>>,
    {
        self.args = args.into();
        self.lua.set_app_data(self.args.clone());
        self
    }

    /**
        Runs a Lune script inside of the current runtime.
    */
    pub async fn run(
        &mut self,
        script_name: impl AsRef<str>,
        script_contents: impl AsRef<[u8]>,
    ) -> Result<ExitCode, LuneError> {
        let main = self
            .lua
            .load(script_contents.as_ref())
            .set_name(script_name.as_ref());

        self.scheduler.push_back(self.lua, main, ())?;

        Ok(self.scheduler.run_to_completion(self.lua).await)
    }

    /**
        Creates a context struct that can be called / ran multiple times,
        preserving the function environment / context between each run.

        Note that this is slightly slower than using [`run`] directly.
    */
    pub fn context(&self, script_name: impl Into<String>) -> Result<LuneContext, LuneError> {
        let script_name = script_name.into();

        let environment = self.lua.create_table()?;
        for pair in self.lua.globals().pairs::<LuaValue, LuaValue>() {
            let (key, value) = pair?;
            environment.set(key, value)?;
        }

        Ok(LuneContext {
            parent: self,
            script_name,
            environment,
        })
    }
}

pub struct LuneContext<'a> {
    parent: &'a Lune,
    script_name: String,
    environment: LuaTable<'a>,
}

impl<'a> LuneContext<'a> {
    /**
        Runs a Lune script inside of the current runtime.

        The function environment / context will be preserved between each run.
    */
    pub async fn run(&mut self, script_contents: impl AsRef<[u8]>) -> Result<ExitCode, LuneError> {
        let main = self
            .parent
            .lua
            .load(script_contents.as_ref())
            .set_name(&self.script_name)
            .set_environment(self.environment.clone());

        self.parent.scheduler.push_back(self.parent.lua, main, ())?;

        Ok(self
            .parent
            .scheduler
            .run_to_completion(self.parent.lua)
            .await)
    }
}
