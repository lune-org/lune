use std::{collections::HashSet, process::ExitCode, sync::Arc};

use mlua::prelude::*;
use tokio::{sync::mpsc, task};

pub(crate) mod globals;
pub(crate) mod lua;
pub(crate) mod utils;

#[cfg(test)]
mod tests;

use crate::utils::{formatting::pretty_format_luau_error, message::LuneMessage};

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

        Some Lune globals such as [`LuneGlobal::Process`] may spawn
        separate tokio tasks on other threads, but the Luau environment
        itself is guaranteed to run on a single thread in the local set.

        Note that this will create a static Lua instance that will live
        for the remainer of the program, and that this leaks memory using
        [`Box::leak`] that will then get deallocated when the program exits.
    */
    pub async fn run(
        &self,
        script_name: &str,
        script_contents: &str,
    ) -> Result<ExitCode, LuaError> {
        let task_set = task::LocalSet::new();
        let (sender, mut receiver) = mpsc::channel::<LuneMessage>(64);
        let lua = Lua::new().into_static();
        let snd = Arc::new(sender);
        lua.set_app_data(Arc::downgrade(&snd));
        // Add in wanted lune globals
        for global in self.includes.clone() {
            if !self.excludes.contains(&global) {
                global.inject(lua)?;
            }
        }
        // Spawn the main thread from our entrypoint script
        let script_name = script_name.to_string();
        let script_chunk = script_contents.to_string();
        let script_sender = snd.clone();
        script_sender
            .send(LuneMessage::Spawned)
            .await
            .map_err(LuaError::external)?;
        task_set.spawn_local(async move {
            let result = lua
                .load(&script_chunk)
                .set_name(&format!("={script_name}"))
                .unwrap()
                .eval_async::<LuaValue>()
                .await;
            match result {
                Err(e) => script_sender.send(LuneMessage::LuaError(e)).await,
                Ok(_) => script_sender.send(LuneMessage::Finished).await,
            }
        });
        // Run the executor until there are no tasks left,
        // taking care to not exit right away for errors
        let (got_code, got_error, exit_code) = task_set
            .run_until(async {
                let mut task_count = 0;
                let mut got_error = false;
                let mut got_code = false;
                let mut exit_code = 0;
                while let Some(message) = receiver.recv().await {
                    // Make sure our task-count-modifying messages are sent correctly, one
                    // task spawned must always correspond to one task finished / errored
                    match &message {
                        LuneMessage::Exit(_) => {}
                        LuneMessage::Spawned => {}
                        message => {
                            if task_count == 0 {
                                return Err(format!(
                                    "Got message while task count was 0!\nMessage: {message:#?}"
                                ));
                            }
                        }
                    }
                    // Handle whatever message we got
                    match message {
                        LuneMessage::Exit(code) => {
                            exit_code = code;
                            got_code = true;
                            break;
                        }
                        LuneMessage::Spawned => task_count += 1,
                        LuneMessage::Finished => task_count -= 1,
                        LuneMessage::LuaError(e) => {
                            eprintln!("{}", pretty_format_luau_error(&e));
                            got_error = true;
                            task_count -= 1;
                        }
                    };
                    // If there are no tasks left running, it is now
                    // safe to close the receiver and end execution
                    if task_count == 0 {
                        receiver.close();
                    }
                }
                Ok((got_code, got_error, exit_code))
            })
            .await
            .map_err(LuaError::external)?;
        // If we got an error, we will default to exiting
        // with code 1, unless a code was manually given
        if got_code {
            Ok(ExitCode::from(exit_code))
        } else if got_error {
            Ok(ExitCode::FAILURE)
        } else {
            Ok(ExitCode::SUCCESS)
        }
    }
}
