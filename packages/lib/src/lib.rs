use std::{collections::HashSet, process::ExitCode};

use lua::task::TaskScheduler;
use mlua::prelude::*;
use tokio::task::LocalSet;

pub(crate) mod globals;
pub(crate) mod lua;
pub(crate) mod utils;

#[cfg(test)]
mod tests;

use crate::utils::formatting::pretty_format_luau_error;

pub use globals::LuneGlobal;

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
        let lua = Lua::new().into_static();
        // Store original lua global functions in the registry so we can use
        // them later without passing them around and dealing with lifetimes
        lua.set_named_registry_value("require", lua.globals().get::<_, LuaFunction>("require")?)?;
        lua.set_named_registry_value("print", lua.globals().get::<_, LuaFunction>("print")?)?;
        lua.set_named_registry_value("error", lua.globals().get::<_, LuaFunction>("error")?)?;
        let coroutine: LuaTable = lua.globals().get("coroutine")?;
        lua.set_named_registry_value("co.thread", coroutine.get::<_, LuaFunction>("running")?)?;
        lua.set_named_registry_value("co.yield", coroutine.get::<_, LuaFunction>("yield")?)?;
        lua.set_named_registry_value("co.close", coroutine.get::<_, LuaFunction>("close")?)?;
        let debug: LuaTable = lua.globals().raw_get("debug")?;
        lua.set_named_registry_value("dbg.info", debug.get::<_, LuaFunction>("info")?)?;
        // Create our task scheduler and schedule the main thread on it
        let sched = TaskScheduler::new(lua)?.into_static();
        lua.set_app_data(sched);
        sched.schedule_next(
            LuaValue::Function(
                lua.load(script_contents)
                    .set_name(script_name)
                    .unwrap()
                    .into_function()
                    .unwrap(),
            ),
            LuaValue::Nil.to_lua_multi(lua)?,
        )?;
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
                    // println!("{result}");
                    if let Some(err) = result.get_lua_error() {
                        eprintln!("{}", pretty_format_luau_error(&err));
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
