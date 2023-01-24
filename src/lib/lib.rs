use std::{collections::HashSet, process::ExitCode, sync::Arc};

use anyhow::{anyhow, bail, Result};
use mlua::prelude::*;
use smol::LocalExecutor;

pub mod globals;
pub mod utils;

use crate::{
    globals::{create_console, create_fs, create_net, create_process, create_require, create_task},
    utils::formatting::pretty_format_luau_error,
};

#[cfg(not(test))]
use crate::utils::formatting::format_label;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LuneGlobal {
    Console,
    Fs,
    Net,
    Process,
    Require,
    Task,
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
        ]
    }
}

#[derive(Debug)]
pub(crate) enum LuneMessage {
    Exit(u8),
    Spawned,
    Finished,
    Error(anyhow::Error),
    LuaError(mlua::Error),
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

    pub async fn run(&self, name: &str, chunk: &str) -> Result<ExitCode> {
        let (s, r) = smol::channel::unbounded::<LuneMessage>();
        let lua = Arc::new(mlua::Lua::new());
        let exec = Arc::new(LocalExecutor::new());
        let sender = Arc::new(s);
        let receiver = Arc::new(r);
        lua.set_app_data(Arc::downgrade(&lua));
        lua.set_app_data(Arc::downgrade(&exec));
        lua.set_app_data(Arc::downgrade(&sender));
        lua.set_app_data(Arc::downgrade(&receiver));
        // Add in wanted lune globals
        for global in &self.globals {
            match &global {
                LuneGlobal::Console => create_console(&lua)?,
                LuneGlobal::Fs => create_fs(&lua)?,
                LuneGlobal::Net => create_net(&lua)?,
                LuneGlobal::Process => create_process(&lua, self.args.clone())?,
                LuneGlobal::Require => create_require(&lua)?,
                LuneGlobal::Task => create_task(&lua)?,
            }
        }
        // Spawn the main thread from our entrypoint script
        let script_name = name.to_string();
        let script_chunk = chunk.to_string();
        exec.spawn(async move {
            sender.send(LuneMessage::Spawned).await?;
            let result = lua
                .load(&script_chunk)
                .set_name(&format!("={}", script_name))
                .unwrap()
                .call_async::<_, LuaMultiValue>(LuaMultiValue::new())
                .await;
            let message = match result {
                Ok(_) => LuneMessage::Finished,
                #[cfg(test)]
                Err(e) => LuneMessage::Error(anyhow!("{}", pretty_format_luau_error(&e))),
                #[cfg(not(test))]
                Err(e) => LuneMessage::Error(anyhow!(
                    "\n{}\n{}",
                    format_label("ERROR"),
                    pretty_format_luau_error(&e)
                )),
            };
            sender.send(message).await
        })
        .detach();
        // Run the executor until there are no tasks left,
        // taking care to not exit right away for errors
        let (got_code, got_error, exit_code) = exec
            .run(async {
                let mut task_count = 0;
                let mut got_error = false;
                let mut got_code = false;
                let mut exit_code = 0;
                while let Ok(message) = receiver.recv().await {
                    // Make sure our task-count-modifying messages are sent correctly, one
                    // task spawned must always correspond to one task finished / errored
                    match &message {
                        LuneMessage::Exit(_) => {}
                        LuneMessage::Spawned => {}
                        message => {
                            if task_count == 0 {
                                bail!(
                                    "Got message while task count was 0!\nMessage: {:#?}",
                                    message
                                )
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
                        LuneMessage::Error(e) => {
                            eprintln!("{}", e);
                            got_error = true;
                            task_count += 1;
                        }
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
            .await?;
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
    use std::process::ExitCode;

    use anyhow::Result;
    use smol::fs::read_to_string;

    use crate::Lune;

    const ARGS: &[&str] = &["Foo", "Bar"];

    macro_rules! run_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() -> Result<ExitCode> {
                    smol::block_on(async {
                        let full_name = format!("src/tests/{}.luau", $value);
                        let script = read_to_string(&full_name)
                            .await
                            .unwrap();
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
                        lune.run(&script_name, &script).await
                    })
                }
            )*
        }
    }

    run_tests! {
        console_format: "console/format",
        console_set_color: "console/set_color",
        console_set_style: "console/set_style",
        fs_files: "fs/files",
        fs_dirs: "fs/dirs",
        net_request_codes: "net/request/codes",
        net_request_methods: "net/request/methods",
        net_request_redirect: "net/request/redirect",
        net_json_decode: "net/json/decode",
        net_json_encode: "net/json/encode",
        process_args: "process/args",
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
