#![allow(clippy::missing_panics_doc)]

use std::{
    ffi::OsString,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

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
        Runs a script that represents custom input, inside of the current runtime.

        # Errors

        Returns an error if the script fails to run, but not if the script itself errors.
    */
    pub async fn run_custom(
        &mut self,
        script_name: impl AsRef<str>,
        script_contents: impl AsRef<[u8]>,
    ) -> RuntimeResult<RuntimeReturnValues> {
        let script_name = format!("={}", script_name.as_ref());
        self.run(script_name, script_contents).await
    }

    /**
        Runs a script that represents a file, inside of the current runtime.

        It is important that the given `script_path` represents a real file path
        for require calls to resolve properly - otherwise, use `run_custom`.

        # Errors

        Returns an error if the script fails to run, but not if the script itself errors.
    */
    pub async fn run_file(
        &mut self,
        script_path: impl AsRef<str>,
        script_contents: impl AsRef<[u8]>,
    ) -> RuntimeResult<RuntimeReturnValues> {
        let script_name = format!("@{}", script_path.as_ref());
        self.run(script_name, script_contents).await
    }

    async fn run(
        &mut self,
        script_name: impl AsRef<str>,
        script_contents: impl AsRef<[u8]>,
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
            .load(script_contents.as_ref())
            .set_name(script_name.as_ref());

        // Run it on our scheduler until it and any other spawned threads complete
        let main_thread_id = self.sched.push_thread_back(main, ())?;
        self.sched.run().await;

        let main_thread_values = match self.sched.get_thread_result(main_thread_id) {
            Some(res) => res,
            None => LuaValue::Nil.into_lua_multi(&self.lua),
        }?;

        Ok(RuntimeReturnValues {
            code: self.sched.get_exit_code(),
            errored: got_any_error.load(Ordering::SeqCst),
            values: main_thread_values,
        })
    }
}
