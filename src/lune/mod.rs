use std::{process::ExitCode, sync::Arc};

use mlua::prelude::*;

mod error;
mod scheduler;

use self::scheduler::Scheduler;

pub use error::LuneError;

#[derive(Clone, Debug, Default)]
pub struct Lune {
    args: Vec<String>,
}

impl Lune {
    /**
        Creates a new Lune script runner.
    */
    pub fn new() -> Self {
        Self::default()
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
        let lua = Arc::new(Lua::new());
        let sched = Scheduler::new(Arc::clone(&lua));

        let main_fn = lua
            .load(script_contents.as_ref())
            .set_name(script_name.as_ref())
            .into_function()?;
        let main_thread = lua.create_thread(main_fn)?.into_owned();

        sched
            .push_back(main_thread, ())
            .expect("Failed to enqueue thread for main");
        Ok(sched.run_to_completion().await)
    }
}
