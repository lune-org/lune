use std::process::ExitCode;

mod error;
mod scheduler;

use self::scheduler::Scheduler;

pub use error::LuneError;
use mlua::Lua;

#[derive(Debug, Clone)]
pub struct Lune {
    lua: &'static Lua,
    args: Vec<String>,
}

impl Lune {
    /**
        Creates a new Lune script runner.
    */
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            lua: Lua::new().into_static(),
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
        self
    }

    /**
        Runs a Lune script inside of a new Luau VM.
    */
    pub async fn run(
        &self,
        script_name: impl AsRef<str>,
        script_contents: impl AsRef<[u8]>,
    ) -> Result<ExitCode, LuneError> {
        let scheduler = Scheduler::new(self.lua);
        self.lua.set_app_data(scheduler.clone());

        let main = self
            .lua
            .load(script_contents.as_ref())
            .set_name(script_name.as_ref());

        scheduler.push_back(main, ())?;
        Ok(scheduler.run_to_completion().await)
    }
}

impl Drop for Lune {
    fn drop(&mut self) {
        // SAFETY: The scheduler needs the static lifetime reference to lua,
        // when dropped nothing outside of here has access to the scheduler
        unsafe {
            Lua::from_static(self.lua);
        }
    }
}
