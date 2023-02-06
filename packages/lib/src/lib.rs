use std::{collections::HashSet, process::ExitCode, sync::Arc};

use mlua::prelude::*;
use tokio::{sync::mpsc, task};

pub(crate) mod globals;
pub(crate) mod utils;

use crate::{
    globals::{
        create_console, create_fs, create_net, create_process, create_require, create_task,
        create_top_level,
    },
    utils::formatting::pretty_format_luau_error,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LuneGlobal {
    Console,
    Fs,
    Net,
    Process,
    Require,
    Task,
    TopLevel,
}

impl LuneGlobal {
    pub fn get_all() -> Vec<Self> {
        vec![
            Self::Console,
            Self::Fs,
            Self::Net,
            Self::Process,
            Self::Require,
            Self::Task,
            Self::TopLevel,
        ]
    }
}

#[derive(Debug)]
pub(crate) enum LuneMessage {
    Exit(u8),
    Spawned,
    Finished,
    LuaError(LuaError),
}

#[derive(Clone, Debug, Default)]
pub struct Lune {
    globals: HashSet<LuneGlobal>,
    args: Vec<String>,
}

impl Lune {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_global(mut self, global: LuneGlobal) -> Self {
        self.globals.insert(global);
        self
    }

    pub fn with_all_globals(mut self) -> Self {
        for global in LuneGlobal::get_all() {
            self.globals.insert(global);
        }
        self
    }

    pub async fn run(&self, name: &str, chunk: &str) -> Result<ExitCode, LuaError> {
        let task_set = task::LocalSet::new();
        let (sender, mut receiver) = mpsc::channel::<LuneMessage>(64);
        let lua = Arc::new(mlua::Lua::new());
        let snd = Arc::new(sender);
        lua.set_app_data(Arc::downgrade(&lua));
        lua.set_app_data(Arc::downgrade(&snd));
        // Add in wanted lune globals
        for global in &self.globals {
            match &global {
                LuneGlobal::Console => create_console(&lua)?,
                LuneGlobal::Fs => create_fs(&lua)?,
                LuneGlobal::Net => create_net(&lua)?,
                LuneGlobal::Process => create_process(&lua, self.args.clone())?,
                LuneGlobal::Require => create_require(&lua)?,
                LuneGlobal::Task => create_task(&lua)?,
                LuneGlobal::TopLevel => create_top_level(&lua)?,
            }
        }
        // Spawn the main thread from our entrypoint script
        let script_lua = lua.clone();
        let script_name = name.to_string();
        let script_chunk = chunk.to_string();
        let script_sender = snd.clone();
        script_sender
            .send(LuneMessage::Spawned)
            .await
            .map_err(LuaError::external)?;
        task_set.spawn_local(async move {
            let result = script_lua
                .load(&script_chunk)
                .set_name(&format!("={}", script_name))
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
                                    "Got message while task count was 0!\nMessage: {:#?}",
                                    message
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
                            task_count += 1;
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

#[cfg(test)]
mod tests {
    use std::{env::set_current_dir, path::PathBuf, process::ExitCode};

    use anyhow::Result;
    use tokio::fs::read_to_string;

    use crate::Lune;

    const ARGS: &[&str] = &["Foo", "Bar"];

    macro_rules! run_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[tokio::test]
                async fn $name() -> Result<ExitCode> {
                    // NOTE: This path is relative to the lib
                    // package, not the cwd or workspace root,
                    // so we need to cd to the repo root first
                    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                    let root_dir = crate_dir.join("../../").canonicalize()?;
                    set_current_dir(root_dir)?;
                    // The rest of the test logic can continue as normal
                    let full_name = format!("tests/{}.luau", $value);
                    let script = read_to_string(&full_name).await?;
                    let lune = Lune::new()
                        .with_args(
                            ARGS
                                .clone()
                                .iter()
                                .map(ToString::to_string)
                                .collect()
                        )
                        .with_all_globals();
                    let script_name = full_name.strip_suffix(".luau").unwrap();
                    let exit_code = lune.run(&script_name, &script).await?;
                    Ok(exit_code)
                }
            )*
        }
    }

    run_tests! {
        console_format: "console/format",
        console_set_style: "console/set_style",
        fs_files: "fs/files",
        fs_dirs: "fs/dirs",
        net_request_codes: "net/request/codes",
        net_request_methods: "net/request/methods",
        net_request_redirect: "net/request/redirect",
        net_json_decode: "net/json/decode",
        net_json_encode: "net/json/encode",
        net_serve: "net/serve",
        process_args: "process/args",
        process_cwd: "process/cwd",
        process_env: "process/env",
        process_exit: "process/exit",
        process_spawn: "process/spawn",
        require_children: "require/tests/children",
        require_invalid: "require/tests/invalid",
        require_nested: "require/tests/nested",
        require_parents: "require/tests/parents",
        require_siblings: "require/tests/siblings",
        task_cancel: "task/cancel",
        task_defer: "task/defer",
        task_delay: "task/delay",
        task_spawn: "task/spawn",
        task_wait: "task/wait",
    }
}
