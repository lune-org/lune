use anyhow::Result;
use mlua::Lua;

pub mod globals;
pub mod utils;

use crate::{
    globals::{Console, Fs, Net, Process},
    utils::formatting::{pretty_print_luau_error, print_label},
};

pub struct Lune {
    lua: Lua,
    args: Vec<String>,
}

impl Lune {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        lua.sandbox(true)?;
        Ok(Self { lua, args: vec![] })
    }

    pub fn with_args(mut self, args: Vec<String>) -> Result<Self> {
        self.args = args;
        Ok(self)
    }

    pub fn with_default_globals(self) -> Result<Self> {
        {
            let globals = self.lua.globals();
            globals.set("console", Console::new())?;
            globals.set("fs", Fs())?;
            globals.set("net", Net::new())?;
            globals.set("process", Process::new(self.args.clone()))?;
        }
        Ok(self)
    }

    pub async fn run(&self, chunk: &str) -> Result<()> {
        self.handle_result(self.lua.load(chunk).exec_async().await)
    }

    pub async fn run_with_name(&self, chunk: &str, name: &str) -> Result<()> {
        self.handle_result(self.lua.load(chunk).set_name(name)?.exec_async().await)
    }

    fn handle_result(&self, result: mlua::Result<()>) -> Result<()> {
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!();
                print_label("ERROR").unwrap();
                eprintln!();
                pretty_print_luau_error(&e);
                Err(e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
                    let lune = crate::Lune::new()
                        .unwrap()
                        .with_args(args)
                        .unwrap()
                        .with_default_globals()
                        .unwrap();
                    let script = tokio::fs::read_to_string(&path)
                        .await
                        .unwrap();
                    if let Err(e) = lune.run_with_name(&script, $value).await {
                        panic!("Test '{}' failed!\n{}", $value, e.to_string())
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
        process_spawn: "process/spawn",
        net_request_codes: "net/request/codes",
        net_request_methods: "net/request/methods",
        net_request_redirect: "net/request/redirect",
        net_json_decode: "net/json/decode",
        net_json_encode: "net/json/encode",
    }
}
