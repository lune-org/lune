use std::{collections::HashSet, process::ExitCode};

use lua::task::{TaskScheduler, TaskSchedulerResumeExt, TaskSchedulerScheduleExt};
use mlua::prelude::*;
use tokio::task::LocalSet;

pub(crate) mod globals;
pub(crate) mod lua;
pub(crate) mod utils;

#[cfg(test)]
mod tests;

use crate::utils::formatting::pretty_format_luau_error;

pub use globals::LuneGlobal;
pub use lua::create_lune_lua;

#[derive(Clone, Debug, Default)]
pub struct Lune {
    includes: HashSet<LuneGlobal>,
    excludes: HashSet<LuneGlobal>,
}

impl Lune {
    /**
        Creates a new Lune script runner.
    */
    pub fn new() -> Self {
        Self::default()
    }

    /**
        Include a global in the lua environment created for running a Lune script.
    */
    pub fn with_global(mut self, global: LuneGlobal) -> Self {
        self.includes.insert(global);
        self
    }

    /**
        Include all globals in the lua environment created for running a Lune script.
    */
    pub fn with_all_globals(mut self) -> Self {
        for global in LuneGlobal::all::<String>(&[]) {
            self.includes.insert(global);
        }
        self
    }

    /**
        Include all globals in the lua environment created for running a
        Lune script, as well as supplying args for [`LuneGlobal::Process`].
    */
    pub fn with_all_globals_and_args(mut self, args: Vec<String>) -> Self {
        for global in LuneGlobal::all(&args) {
            self.includes.insert(global);
        }
        self
    }

    /**
        Exclude a global from the lua environment created for running a Lune script.

        This should be preferred over manually iterating and filtering
        which Lune globals to add to the global environment.
    */
    pub fn without_global(mut self, global: LuneGlobal) -> Self {
        self.excludes.insert(global);
        self
    }

    /**
        Runs a Lune script.

        This will create a new sandboxed Luau environment with the configured
        globals and arguments, running inside of a [`tokio::task::LocalSet`].

        Some Lune globals such as [`LuneGlobal::Process`] and [`LuneGlobal::Net`]
        may spawn separate tokio tasks on other threads, but the Luau environment
        itself is guaranteed to run on a single thread in the local set.

        Note that this will create a static Lua instance and task scheduler which both
        will live for the remainer of the program, and that this leaks memory using
        [`Box::leak`] that will then get deallocated when the program exits.
    */
    pub async fn run(
        &self,
        script_name: &str,
        script_contents: &str,
    ) -> Result<ExitCode, LuaError> {
        // Create our special lune-flavored Lua object with extra registry values
        let lua = create_lune_lua().expect("Failed to create Lua object");
        // Create our task scheduler
        let sched = TaskScheduler::new(lua)?.into_static();
        lua.set_app_data(sched);
        // Create the main thread and schedule it
        let main_chunk = lua
            .load(script_contents)
            .set_name(script_name)
            .unwrap()
            .into_function()
            .unwrap();
        let main_thread = lua.create_thread(main_chunk).unwrap();
        let main_thread_args = LuaValue::Nil.to_lua_multi(lua)?;
        sched.schedule_blocking(main_thread, main_thread_args)?;
        // Create our wanted lune globals, some of these need
        // the task scheduler be available during construction
        for global in self.includes.clone() {
            if !self.excludes.contains(&global) {
                global.inject(lua)?;
            }
        }
        // Keep running the scheduler until there are either no tasks
        // left to run, or until a task requests to exit the process
        let exit_code = LocalSet::new()
            .run_until(async move {
                let mut got_error = false;
                loop {
                    let result = sched.resume_queue().await;
                    if let Some(err) = result.get_lua_error() {
                        eprintln!("{}", pretty_format_luau_error(&err, true));
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
