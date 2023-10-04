use std::process::ExitCode;

use mlua::Lua;

mod builtins;
mod error;
mod globals;
mod scheduler;

pub(crate) mod util;

use self::scheduler::{LuaSchedulerExt, Scheduler};

pub use error::LuneError;

// TODO: Rename this struct to "Runtime" instead for the
// next breaking release, it's a more fitting name and
// will probably be more obvious when browsing files
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
        /*
            FUTURE: Stop leaking these when we have removed the lifetime
            on the scheduler and can place them in lua app data using arc

            See the scheduler struct for more notes
        */
        let lua = Lua::new().into_static();
        let scheduler = Scheduler::new().into_static();

        lua.set_scheduler(scheduler);
        lua.set_app_data(Vec::<String>::new());
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

        This will preserve any modifications to global values / context.
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
}
