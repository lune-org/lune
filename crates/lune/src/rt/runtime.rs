#![allow(clippy::missing_panics_doc)]

use std::{
    process::ExitCode,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use mlua::prelude::*;
use mlua_luau_scheduler::{Functions, Scheduler};

use super::{RuntimeError, RuntimeResult};

#[derive(Debug)]
pub struct Runtime {
    lua: Rc<Lua>,
    args: Vec<String>,
}

impl Runtime {
    /**
        Creates a new Lune runtime, with a new Luau VM.

        Injects standard globals and libraries if any of the `std` features are enabled.
    */
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let lua = Rc::new(Lua::new());

        lua.set_app_data(Rc::downgrade(&lua));
        lua.set_app_data(Vec::<String>::new());

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
            lune_std::inject_globals(&lua).expect("Failed to inject globals");
        }

        Self {
            lua,
            args: Vec::new(),
        }
    }

    /**
        Sets arguments to give in `process.args` for Lune scripts.
    */
    #[must_use]
    pub fn with_args<V>(mut self, args: V) -> Self
    where
        V: Into<Vec<String>>,
    {
        self.args = args.into();
        self.lua.set_app_data(self.args.clone());
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
    ) -> RuntimeResult<ExitCode> {
        // Create a new scheduler for this run
        let sched = Scheduler::new(&self.lua);

        // Add error callback to format errors nicely + store status
        let got_any_error = Arc::new(AtomicBool::new(false));
        let got_any_inner = Arc::clone(&got_any_error);
        sched.set_error_callback(move |e| {
            got_any_inner.store(true, Ordering::SeqCst);
            eprintln!("{}", RuntimeError::from(e));
        });

        // Overwrite resume & wrap functions on the coroutine global
        // with ones that are compatible with our scheduler
        // We also sandbox the VM, preventing further modifications
        // to the global environment, and enabling optimizations
        inject_scheduler_functions_and_sandbox(&self.lua)?;

        // Load our "main" thread
        let main = self
            .lua
            .load(script_contents.as_ref())
            .set_name(script_name.as_ref());

        // Run it on our scheduler until it and any other spawned threads complete
        sched.push_thread_back(main, ())?;
        sched.run().await;

        // Return the exit code - default to FAILURE if we got any errors
        let exit_code = sched.get_exit_code().unwrap_or({
            if got_any_error.load(Ordering::SeqCst) {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        });

        Ok(exit_code)
    }
}

fn inject_scheduler_functions_and_sandbox(lua: &Lua) -> LuaResult<()> {
    let fns = Functions::new(lua)?;

    let co = lua.globals().get::<_, LuaTable>("coroutine")?;
    co.set("resume", fns.resume.clone())?;
    co.set("wrap", fns.wrap.clone())?;

    lua.sandbox(true)?;

    // NOTE: We need to create the _G table after
    // sandboxing, otherwise it will be read-only
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
        let g_global = lune_std::LuneStandardGlobal::GTable;
        lua.globals().set(g_global.name(), g_global.create(lua)?)?;
    }

    Ok(())
}
