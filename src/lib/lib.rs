use std::collections::HashSet;

use anyhow::{bail, Result};
use mlua::Lua;
use tokio::task;

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
        let run_name = name.to_owned();
        let run_chunk = chunk.to_owned();
        let run_globals = self.globals.to_owned();
        let run_args = self.args.to_owned();
        // Spawn a thread-local task so that we can then spawn
        // more tasks in our globals without the Send requirement
        let local = task::LocalSet::new();
        local
            .run_until(async move {
                task::spawn_local(async move {
                    let lua = Lua::new();
                    for global in &run_globals {
                        match &global {
                            LuneGlobal::Console => create_console(&lua).await?,
                            LuneGlobal::Fs => create_fs(&lua).await?,
                            LuneGlobal::Net => create_net(&lua).await?,
                            LuneGlobal::Process => create_process(&lua, run_args.clone()).await?,
                            LuneGlobal::Task => create_task(&lua).await?,
                        }
                    }
                    let result = lua.load(&run_chunk).set_name(&run_name)?.exec_async().await;
                    match result {
                        Ok(_) => Ok(()),
                        Err(e) => bail!(
                            "\n{}\n{}",
                            format_label("ERROR"),
                            pretty_format_luau_error(&e)
                        ),
                    }
                })
                .await
                .unwrap()
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::Lune;

    macro_rules! run_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[tokio::test]
                async fn $name() {
                    let args = vec![
                        "Foo".to_owned(),
                        "Bar".to_owned()
                    ];
                    let path = std::env::current_dir()
                        .unwrap()
                        .join(format!("src/tests/{}.luau", $value));
                    let script = tokio::fs::read_to_string(&path)
                        .await
                        .unwrap();
                    let lune = Lune::new()
                        .with_args(args)
                        .with_all_globals();
                    if let Err(e) = lune.run($value, &script).await {
                        panic!("\nTest '{}' failed!\n{}\n", $value, e.to_string())
                    }
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
