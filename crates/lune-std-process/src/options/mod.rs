use std::{
    collections::HashMap,
    env::{self},
    path::PathBuf,
};

use mlua::prelude::*;

use async_process::Command;
use directories::UserDirs;

mod kind;
mod stdio;

pub(super) use kind::*;
pub(super) use stdio::*;

#[derive(Debug, Clone, Default)]
pub(super) struct ProcessSpawnOptions {
    pub cwd: Option<PathBuf>,
    pub envs: HashMap<String, String>,
    pub shell: Option<String>,
    pub stdio: ProcessSpawnOptionsStdio,
}

impl FromLua for ProcessSpawnOptions {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        let mut this = Self::default();
        let value = match value {
            LuaValue::Nil => return Ok(this),
            LuaValue::Table(t) => t,
            _ => {
                return Err(LuaError::FromLuaConversionError {
                    from: value.type_name(),
                    to: "ProcessSpawnOptions".to_string(),
                    message: Some(format!(
                        "Invalid spawn options - expected table, got {}",
                        value.type_name()
                    )),
                })
            }
        };

        /*
            If we got a working directory to use:

            1. Substitute leading tilde (~) for the users home dir
            2. Make sure it exists
        */
        match value.get("cwd")? {
            LuaValue::Nil => {}
            LuaValue::String(s) => {
                let mut cwd = PathBuf::from(s.to_str()?.to_string());
                if let Ok(stripped) = cwd.strip_prefix("~") {
                    let user_dirs = UserDirs::new().ok_or_else(|| {
                        LuaError::runtime(
                            "Invalid value for option 'cwd' - failed to get home directory",
                        )
                    })?;
                    cwd = user_dirs.home_dir().join(stripped);
                }
                if !cwd.exists() {
                    return Err(LuaError::runtime(
                        "Invalid value for option 'cwd' - path does not exist",
                    ));
                }
                this.cwd = Some(cwd);
            }
            value => {
                return Err(LuaError::RuntimeError(format!(
                    "Invalid type for option 'cwd' - expected string, got '{}'",
                    value.type_name()
                )))
            }
        }

        /*
            If we got environment variables, make sure they are strings
        */
        match value.get("env")? {
            LuaValue::Nil => {}
            LuaValue::Table(e) => {
                for pair in e.pairs::<String, String>() {
                    let (k, v) = pair.context("Environment variables must be strings")?;
                    this.envs.insert(k, v);
                }
            }
            value => {
                return Err(LuaError::RuntimeError(format!(
                    "Invalid type for option 'env' - expected table, got '{}'",
                    value.type_name()
                )))
            }
        }

        /*
            If we got a shell to use:

            1. When given as a string, use that literally
            2. When set to true, use a default shell for the platform
        */
        match value.get("shell")? {
            LuaValue::Nil => {}
            LuaValue::String(s) => this.shell = Some(s.to_string_lossy().to_string()),
            LuaValue::Boolean(true) => {
                this.shell = match env::consts::FAMILY {
                    "unix" => Some("/bin/sh".to_string()),
                    "windows" => Some("powershell".to_string()),
                    _ => None,
                };
            }
            value => {
                return Err(LuaError::RuntimeError(format!(
                    "Invalid type for option 'shell' - expected 'true' or 'string', got '{}'",
                    value.type_name()
                )))
            }
        }

        /*
            If we got options for stdio handling, parse those as well - note that
            we accept a separate "stdin" value here for compatibility with older
            scripts, but the user should preferrably pass it in the stdio table
        */
        this.stdio = value.get("stdio")?;
        match value.get("stdin")? {
            LuaValue::Nil => {}
            LuaValue::String(s) => this.stdio.stdin = Some(s.as_bytes().to_vec()),
            value => {
                return Err(LuaError::RuntimeError(format!(
                    "Invalid type for option 'stdin' - expected 'string', got '{}'",
                    value.type_name()
                )))
            }
        }

        Ok(this)
    }
}

impl ProcessSpawnOptions {
    pub fn into_command(self, program: impl Into<String>, args: Option<Vec<String>>) -> Command {
        let mut program = program.into();

        // Run a shell using the command param if wanted
        let pargs = match self.shell {
            None => args,
            Some(shell) => {
                let shell_args = match args {
                    Some(args) => vec!["-c".to_string(), format!("{} {}", program, args.join(" "))],
                    None => vec!["-c".to_string(), program.to_string()],
                };
                program = shell.to_string();
                Some(shell_args)
            }
        };

        // Create command with the wanted options
        let mut cmd = match pargs {
            None => Command::new(program),
            Some(args) => {
                let mut cmd = Command::new(program);
                cmd.args(args);
                cmd
            }
        };

        // Set dir to run in and env variables
        if let Some(cwd) = self.cwd {
            cmd.current_dir(cwd);
        }
        if !self.envs.is_empty() {
            cmd.envs(self.envs);
        }

        cmd
    }
}
