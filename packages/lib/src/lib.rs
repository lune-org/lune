use std::process::ExitCode;

use lua::task::{TaskScheduler, TaskSchedulerResumeExt, TaskSchedulerScheduleExt};
use mlua::prelude::*;
use tokio::task::LocalSet;

pub(crate) mod globals;
pub(crate) mod lua;

mod error;
#[cfg(test)]
mod tests;

pub use error::LuneError;
pub use globals::LuneGlobal;
pub use lua::create_lune_lua;

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
        Runs a Lune script.

        This will create a new sandboxed Luau environment with the configured
        globals and arguments, running inside of a [`tokio::task::LocalSet`].

        Some Lune globals may spawn separate tokio tasks on other threads, but the Luau
        environment itself is guaranteed to run on a single thread in the local set.

        Note that this will create a static Lua instance and task scheduler that will
        both live for the remainer of the program, and that this leaks memory using
        [`Box::leak`] that will then get deallocated when the program exits.
    */
    pub async fn try_run(
        &self,
        script_name: impl AsRef<str>,
        script_contents: impl AsRef<[u8]>,
    ) -> Result<ExitCode, LuneError> {
        self.run(script_name, script_contents, true)
            .await
            .map_err(LuneError::from_lua_error)
    }

    /**
        Tries to run a Lune script directly, returning the underlying
        result type if loading the script raises a lua error.

        Passing `false` as the third argument `emit_prettified_errors` will
        bypass any additional error formatting automatically done by Lune
        for errors that are emitted while the script is running.

        Behavior is otherwise exactly the same as `try_run`
        and `try_run` should be preferred in all other cases.
    */
    #[doc(hidden)]
    pub async fn run(
        &self,
        script_name: impl AsRef<str>,
        script_contents: impl AsRef<[u8]>,
        emit_prettified_errors: bool,
    ) -> Result<ExitCode, LuaError> {
        // Create our special lune-flavored Lua object with extra registry values
        let lua = create_lune_lua()?;
        // Create our task scheduler
        let sched = TaskScheduler::new(lua)?.into_static();
        lua.set_app_data(sched);
        // Create the main thread and schedule it
        let main_chunk = lua
            .load(script_contents.as_ref())
            .set_name(script_name.as_ref())?
            .into_function()?;
        let main_thread = lua.create_thread(main_chunk)?;
        let main_thread_args = LuaValue::Nil.to_lua_multi(lua)?;
        sched.schedule_blocking(main_thread, main_thread_args)?;
        // Create our wanted lune globals, some of these need
        // the task scheduler be available during construction
        for global in LuneGlobal::all(&self.args) {
            global.inject(lua)?;
        }
        // Keep running the scheduler until there are either no tasks
        // left to run, or until a task requests to exit the process
        let exit_code = LocalSet::new()
            .run_until(async move {
                let mut got_error = false;
                loop {
                    let result = sched.resume_queue().await;
                    if let Some(err) = result.get_lua_error() {
                        if emit_prettified_errors {
                            eprintln!("{}", LuneError::from_lua_error(err));
                        } else {
                            eprintln!("{err}");
                        }
                        got_error = true;
                    }
                    if result.is_done() {
                        if let Some(exit_code) = result.get_exit_code() {
                            break exit_code;
                        } else if got_error {
                            break ExitCode::FAILURE;
                        } else {
                            break ExitCode::SUCCESS;
                        }
                    }
                }
            })
            .await;
        Ok(exit_code)
    }
}
