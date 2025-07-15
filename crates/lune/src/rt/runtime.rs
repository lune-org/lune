#![allow(clippy::missing_panics_doc)]

use std::{
    ffi::OsString,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use async_fs as fs;
use lune_utils::process::{ProcessArgs, ProcessEnv, ProcessJitEnablement};
use mlua::prelude::*;
use mlua_luau_scheduler::{Functions, Scheduler};

use super::{RuntimeError, RuntimeResult};

/**
    Values returned by running a Lune runtime until completion.
*/
#[derive(Debug)]
#[non_exhaustive]
pub struct RuntimeReturnValues {
    /// The exit code manually returned from the runtime, if any.
    pub code: Option<u8>,
    /// Whether any errors were thrown from threads
    /// that were not the main thread, or not.
    pub errored: bool,
    /// The final values returned by the main thread.
    pub values: LuaMultiValue,
}

impl RuntimeReturnValues {
    /**
        Returns the final, combined "status" of the runtime return values.

        If no exit code was explicitly set by either the main thread,
        or any threads it may have spawned, the status will be either:

        - `0` if no threads errored
        - `1` if any threads errored
    */
    #[must_use]
    pub fn status(&self) -> u8 {
        self.code.unwrap_or(u8::from(self.errored))
    }

    /**
        Returns whether the run was considered successful, or not.

        See [`RuntimeReturnValues::status`] for more information.
    */
    #[must_use]
    pub fn success(&self) -> bool {
        self.status() == 0
    }
}

/**
    A Lune runtime.
*/
pub struct Runtime {
    lua: Lua,
    sched: Scheduler,
    args: ProcessArgs,
    env: ProcessEnv,
    jit: ProcessJitEnablement,
}

impl Runtime {
    /**
        Creates a new Lune runtime, with a new Luau VM.

        Injects standard globals and libraries if any of the `std` features are enabled.

        # Errors

        - If out of memory or other memory-related errors occur
        - If any of the standard globals and libraries fail to inject
    */
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new();

        let sched = Scheduler::new(lua.clone());
        let fns = Functions::new(lua.clone()).expect("has scheduler");

        // Overwrite some globals that are not compatible with our scheduler
        let co = lua.globals().get::<LuaTable>("coroutine")?;
        co.set("resume", fns.resume.clone())?;
        co.set("wrap", fns.wrap.clone())?;

        // Inject all the globals that are enabled
        #[cfg(any(
            feature = "std-datetime",
            feature = "std-fs",
            feature = "std-luau",
            feature = "std-net",
            feature = "std-process",
            feature = "std-regex",
            feature = "std-roblox",
            feature = "std-serde",
            feature = "std-stdio",
            feature = "std-task",
        ))]
        {
            lune_std::set_global_version(&lua, env!("CARGO_PKG_VERSION"));
            lune_std::inject_globals(lua.clone())?;
        }

        // Sandbox the Luau VM and make it go zooooooooom
        lua.sandbox(true)?;

        // _G table needs to be injected again after sandboxing,
        // otherwise it will be read-only and completely unusable
        #[cfg(any(
            feature = "std-datetime",
            feature = "std-fs",
            feature = "std-luau",
            feature = "std-net",
            feature = "std-process",
            feature = "std-regex",
            feature = "std-roblox",
            feature = "std-serde",
            feature = "std-stdio",
            feature = "std-task",
        ))]
        {
            let g_table = lune_std::LuneStandardGlobal::GTable;
            lua.globals()
                .set(g_table.name(), g_table.create(lua.clone())?)?;
        }

        let args = ProcessArgs::current();
        let env = ProcessEnv::current();
        let jit = ProcessJitEnablement::default();

        Ok(Self {
            lua,
            sched,
            args,
            env,
            jit,
        })
    }

    /**
        Sets arguments to give in `process.args` for Lune scripts.

        By default, `std::env::args_os()` is used.
    */
    #[must_use]
    pub fn with_args<A, S>(mut self, args: A) -> Self
    where
        A: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /**
        Sets environment values to give in `process.env` for Lune scripts.

        By default, `std::env::vars_os()` is used.
    */
    #[must_use]
    pub fn with_env<E, K, V>(mut self, env: E) -> Self
    where
        E: IntoIterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<OsString>,
    {
        self.env = env.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self
    }

    /**
        Enables or disables JIT compilation.
    */
    #[must_use]
    pub fn with_jit<J>(mut self, jit_status: J) -> Self
    where
        J: Into<ProcessJitEnablement>,
    {
        self.jit = jit_status.into();
        self
    }

    /**
        Adds a custom library to the runtime, making it available through `require`.

        # Example Usage

        First, create a library as such:

        ```rs
        Runtime::new().with_lib("@myalias/mylib", |lua| {
            let t = lua.create_table()?;
            let f = lua.create_function(|lua| {
                println!("bar");
                Ok(())
            })?;

            t.set("foo", f)?;

            Ok(t)
        });
        ```

        Then, use it in Lua:

        ```luau
        local lib = require("@myalias/mylib")

        lib.foo() --> "bar"
        ```

        # Errors

        Returns an error if:

        - The library name does not start with `@`
        - The library uses the reserved `lune` alias
        - The library uses the reserved `self` alias
        - The provided `make_lib` function errors
    */
    pub fn with_lib<S, F>(self, name: S, make_lib: F) -> RuntimeResult<Self>
    where
        S: AsRef<str>,
        F: FnOnce(&Lua) -> LuaResult<LuaValue>,
    {
        let name = name.as_ref().trim();

        if !name.starts_with('@') {
            return Err(RuntimeError::from(LuaError::external(
                "Library names must start with '@'",
            )));
        }
        if name.starts_with("@lune/") {
            return Err(RuntimeError::from(LuaError::external(
                "Library names must not start with '@lune/'",
            )));
        }
        if name.starts_with("@self/") {
            return Err(RuntimeError::from(LuaError::external(
                "Library names must not start with '@self/'",
            )));
        }

        let lib = make_lib(&self.lua)?;
        self.lua.register_module(name, lib)?;

        Ok(self)
    }

    /**
        Runs some kind of custom input, inside of the current runtime.

        For any input that is a real file path, [`run_file`] should be used instead.

        # Errors

        Returns an error if:

        - The script fails to run (not if the script itself errors)
    */
    pub async fn run_custom(
        &mut self,
        chunk_name: impl AsRef<str>,
        chunk_contents: impl AsRef<[u8]>,
    ) -> RuntimeResult<RuntimeReturnValues> {
        let chunk_name = format!("={}", chunk_name.as_ref());
        self.run_inner(chunk_name, chunk_contents).await
    }

    /**
        Runs a file at the given file path, inside of the current runtime.

        # Errors

        Returns an error if:

        - The file does not exist or can not be read
        - The script fails to run (not if the script itself errors)
    */
    pub async fn run_file(
        &mut self,
        path: impl Into<PathBuf>,
    ) -> RuntimeResult<RuntimeReturnValues> {
        let path: PathBuf = path.into();
        let contents = fs::read(&path).await.into_lua_err().context(format!(
            "Failed to read file at path \"{}\"",
            path.display()
        ))?;

        // For calls to `require` to resolve properly, we must convert the file
        // path to the respective "module" path according to require-by-string
        let module_path = remove_lua_luau_ext(path);
        let module_name = format!("@{}", module_path.display());
        let module_contents = strip_shebang(contents);

        self.run_inner(module_name, module_contents).await
    }

    async fn run_inner(
        &mut self,
        chunk_name: impl AsRef<str>,
        chunk_contents: impl AsRef<[u8]>,
    ) -> RuntimeResult<RuntimeReturnValues> {
        // Add error callback to format errors nicely + store status
        let got_any_error = Arc::new(AtomicBool::new(false));
        let got_any_inner = Arc::clone(&got_any_error);
        self.sched.set_error_callback(move |e| {
            got_any_inner.store(true, Ordering::SeqCst);
            eprintln!("{}", RuntimeError::from(e));
        });

        // Store the provided args, environment variables, and jit enablement as AppData
        self.lua.set_app_data(self.args.clone());
        self.lua.set_app_data(self.env.clone());
        self.lua.set_app_data(self.jit);

        // Inject all the standard libraries that are enabled - this needs to be done after
        // storing the args/env, since some standard libraries use those during initialization
        #[cfg(any(
            feature = "std-datetime",
            feature = "std-fs",
            feature = "std-luau",
            feature = "std-net",
            feature = "std-process",
            feature = "std-regex",
            feature = "std-roblox",
            feature = "std-serde",
            feature = "std-stdio",
            feature = "std-task",
        ))]
        {
            lune_std::inject_std(self.lua.clone())?;
        }

        // Enable / disable the JIT as requested, before loading anything
        self.lua.enable_jit(self.jit.enabled());

        // Load our "main" thread
        let main = self
            .lua
            .load(chunk_contents.as_ref())
            .set_name(chunk_name.as_ref());

        // Run it on our scheduler until it and any other spawned threads complete
        let main_thread_id = self.sched.push_thread_back(main, ())?;
        self.sched.run().await;

        let main_thread_values = self
            .sched
            .get_thread_result(main_thread_id)
            .unwrap_or_else(|| Ok(LuaMultiValue::new())) // Ignore missing result (interruption), we just want to extract values
            .unwrap_or_default(); // Ignore any errors from the script, we just want to extract values

        Ok(RuntimeReturnValues {
            code: self.sched.get_exit_code(),
            errored: got_any_error.load(Ordering::SeqCst),
            values: main_thread_values,
        })
    }
}

fn remove_lua_luau_ext(path: impl Into<PathBuf>) -> PathBuf {
    let path: PathBuf = path.into();
    match path.extension().and_then(|e| e.to_str()) {
        Some("lua" | "luau") => path.with_extension(""),
        _ => path,
    }
}

fn strip_shebang(mut contents: Vec<u8>) -> Vec<u8> {
    if contents.starts_with(b"#!") {
        if let Some(first_newline_idx) = contents
            .iter()
            .enumerate()
            .find_map(|(idx, c)| if *c == b'\n' { Some(idx) } else { None })
        {
            // NOTE: We keep the newline here on purpose to preserve
            // correct line numbers in stack traces, the only reason
            // we strip the shebang is to get the lua script to parse
            // and the extra newline is not really a problem for that
            contents.drain(..first_newline_idx);
        }
    }
    contents
}
