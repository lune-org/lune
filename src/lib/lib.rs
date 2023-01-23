use std::{collections::HashSet, sync::Arc};

use anyhow::{anyhow, bail, Result};
use mlua::prelude::*;
use smol::LocalExecutor;

pub mod globals;
pub mod utils;

use crate::{
    globals::{create_console, create_fs, create_net, create_process, create_task},
    utils::formatting::{format_label, pretty_format_luau_error},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LuneGlobal {
    Console,
    Fs,
    Net,
    Process,
    Task,
}

impl LuneGlobal {
    pub fn get_all() -> Vec<Self> {
        vec![
            Self::Console,
            Self::Fs,
            Self::Net,
            Self::Process,
            Self::Task,
        ]
    }
}

pub(crate) enum LuneMessage {
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

    pub async fn run(&self, name: &str, chunk: &str) -> Result<()> {
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
                .set_name(&script_name)
                .unwrap()
                .call_async::<_, LuaMultiValue>(LuaMultiValue::new())
                .await;
            let message = match result {
                Ok(_) => LuneMessage::Finished,
                Err(e) => LuneMessage::Error(if cfg!(test) {
                    anyhow!("{}", pretty_format_luau_error(&e))
                } else {
                    anyhow!(
                        "\n{}\n{}",
                        format_label("ERROR"),
                        pretty_format_luau_error(&e)
                    )
                }),
            };
            sender.send(message).await
        })
        .detach();
        // Run the executor until there are no tasks left
        let mut task_count = 1;
        smol::block_on(exec.run(async {
            while let Ok(message) = receiver.recv().await {
                match message {
                    LuneMessage::Spawned => {
                        task_count += 1;
                    }
                    LuneMessage::Finished => {
                        task_count -= 1;
                        if task_count <= 0 {
                            break;
                        }
                    }
                    LuneMessage::Error(e) => {
                        task_count -= 1;
                        bail!("{}", e)
                    }
                    LuneMessage::LuaError(e) => {
                        task_count -= 1;
                        bail!("{}", e)
                    }
                }
            }
            Ok(())
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::Lune;
    use anyhow::Result;
    use smol::fs::read_to_string;
    use std::env::current_dir;

    const ARGS: &[&str] = &["Foo", "Bar"];

    macro_rules! run_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() -> Result<()> {
                    smol::block_on(async {
                        let path = current_dir()
                            .unwrap()
                            .join(format!("src/tests/{}.luau", $value));
                        let script = read_to_string(&path)
                            .await
                            .unwrap();
                        let lune = Lune::new()
                            .with_args(ARGS.clone().iter().map(ToString::to_string).collect())
                            .with_all_globals();
                        lune.run($value, &script).await
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
        process_args: "process/args",
        process_env: "process/env",
        // NOTE: This test does not currently work, it will exit the entire
        // process, meaning it will also exit our test runner and skip testing
        // process_exit: "process/exit",
        process_spawn: "process/spawn",
        net_request_codes: "net/request/codes",
        net_request_methods: "net/request/methods",
        net_request_redirect: "net/request/redirect",
        net_json_decode: "net/json/decode",
        net_json_encode: "net/json/encode",
        task_cancel: "task/cancel",
        task_defer: "task/defer",
        task_delay: "task/delay",
        task_spawn: "task/spawn",
        task_wait: "task/wait",
    }
}
