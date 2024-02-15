use std::{
    process::ExitCode,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use mlua::Lua;
use mlua_luau_scheduler::Scheduler;

mod builtins;
mod error;
mod globals;

pub(crate) mod util;

pub use error::RuntimeError;

#[derive(Debug)]
pub struct Runtime {
    lua: Rc<Lua>,
    args: Vec<String>,
}

impl Runtime {
    /**
        Creates a new Lune runtime, with a new Luau VM.
    */
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let lua = Rc::new(Lua::new());

        lua.set_app_data(Rc::downgrade(&lua));
        lua.set_app_data(Vec::<String>::new());

        Self {
            lua,
            args: Vec::new(),
        }
    }

    /**
        Sets arguments to give in `process.args` for Lune scripts.
    */
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
    */
    pub async fn run(
        &mut self,
        script_name: impl AsRef<str>,
        script_contents: impl AsRef<[u8]>,
    ) -> Result<ExitCode, RuntimeError> {
        // Create a new scheduler for this run
        let sched = Scheduler::new(&self.lua);
        globals::inject_all(&self.lua)?;

        // Add error callback to format errors nicely + store status
        let got_any_error = Arc::new(AtomicBool::new(false));
        let got_any_inner = Arc::clone(&got_any_error);
        sched.set_error_callback(move |e| {
            got_any_inner.store(true, Ordering::SeqCst);
            eprintln!("{}", RuntimeError::from(e));
        });

        // Load our "main" thread
        let main = self
            .lua
            .load(script_contents.as_ref())
            .set_name(script_name.as_ref());

        // Run it on our scheduler until it and any other spawned threads complete
        sched.push_thread_back(main, ())?;
        sched.run().await;

        // Return the exit code - default to FAILURE if we got any errors
        Ok(sched.get_exit_code().unwrap_or({
            if got_any_error.load(Ordering::SeqCst) {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }))
    }
}
