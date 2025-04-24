#![allow(clippy::missing_panics_doc)]

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use lune_utils::jit::JitEnablement;
use mlua::prelude::*;
use mlua_luau_scheduler::{Functions, Scheduler};

use super::{RuntimeError, RuntimeResult};

/**
    Values returned by running a Lune runtime until completion.
*/
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
    jit: JitEnablement,
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

        lua.set_app_data(Vec::<String>::new());

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

        let jit = JitEnablement::default();
        Ok(Self { lua, sched, jit })
    }

    /**
        Sets arguments to give in `process.args` for Lune scripts.
    */
    #[must_use]
    pub fn with_args<A, S>(self, args: A) -> Self
    where
        A: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
        self.lua.set_app_data(args);
        self
    }

    /**
        Enables or disables JIT compilation.
    */
    #[must_use]
    pub fn with_jit(mut self, jit_status: impl Into<JitEnablement>) -> Self {
        self.jit = jit_status.into();
        self
    }

    /**
        Runs a Lune script inside of the current runtime.

        This will preserve any modifications to global values / context.

        # Errors

        This function will return an error if the script fails to run.
    */
    pub async fn run(
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

        // Enable / disable the JIT as requested and store the current status as AppData
        self.lua.set_app_data(self.jit);
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
