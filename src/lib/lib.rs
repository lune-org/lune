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
