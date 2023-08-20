use std::process::ExitCode;

mod builtins;
mod error;
mod globals;
mod scheduler;
mod util;

use self::scheduler::Scheduler;

pub use error::LuneError;
use mlua::Lua;

#[derive(Debug, Clone)]
pub struct Lune {
    lua: &'static Lua,
    scheduler: &'static Scheduler<'static, 'static>,
    args: Vec<String>,
}

impl Lune {
    /**
        Creates a new Lune script runner.
    */
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // FIXME: Leaking these here does not feel great... is there
        // any way for us to create a scheduler, store it in app data, and
        // guarantee it has the same lifetime as Lua without using any unsafe?
        let lua = Lua::new().into_static();
        let scheduler = Scheduler::new(lua).into_static();

        lua.set_app_data(scheduler);
        globals::inject_all(lua).expect("Failed to inject lua globals");

        Self {
            lua,
            scheduler,
            args: Vec::new(),
        }
    }

    /**
        Arguments to give in `process.args` for a Lune script.
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
        Runs a Lune script inside of a new Luau VM.
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

        self.scheduler.push_back(main, ())?;
        Ok(self.scheduler.run_to_completion().await)
    }
}
