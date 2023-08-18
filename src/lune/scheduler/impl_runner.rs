use std::{process::ExitCode, sync::Arc};

use futures_util::StreamExt;
use mlua::prelude::*;

use tokio::task::LocalSet;

use super::{IntoLuaOwnedThread, Scheduler};

const EMPTY_MULTI_VALUE: LuaMultiValue = LuaMultiValue::new();

impl<'lua, 'fut> Scheduler<'fut>
where
    'lua: 'fut,
{
    /**
        Runs all lua threads to completion.

        Returns `true` if any thread was resumed, `false` otherwise.
    */
    fn run_lua_threads(&self) -> bool {
        if self.state.has_exit_code() {
            return false;
        }

        let mut resumed_any = false;

        while let Some((thread, args, sender)) = self
            .pop_thread()
            .expect("Failed to pop thread from scheduler")
        {
            let res = thread.resume::<_, LuaMultiValue>(args);
            self.state.add_resumption();
            resumed_any = true;

            if let Err(err) = &res {
                self.state.add_error();
                eprint!("{err}"); // TODO: Pretty print the lua error here
            }

            if sender.receiver_count() > 0 {
                sender
                    .send(res.map(|v| {
                        Arc::new(
                            self.lua
                                .create_registry_value(v.into_vec())
                                .expect("Failed to store return values in registry"),
                        )
                    }))
                    .expect("Failed to broadcast return values of thread");
            }

            if self.state.has_exit_code() {
                break;
            }
        }

        resumed_any
    }

    /**
        Runs futures until none are left or a future spawned a new lua thread.

        Returns `true` if any future was resumed, `false` otherwise.
    */
    async fn run_futures(&self) -> bool {
        let mut resumed_any = false;

        let mut futs = self
            .futures
            .try_lock()
            .expect("Failed to lock futures queue");
        while futs.next().await.is_some() {
            resumed_any = true;
            if self.has_thread() {
                break;
            }
        }

        resumed_any
    }

    /**
        Runs the scheduler to completion in a [`LocalSet`],
        both normal lua threads and futures, prioritizing
        lua threads over completion of any pending futures.

        Will emit lua output and errors to stdout and stderr.
    */
    pub async fn run_to_completion(&self) -> ExitCode {
        let set = LocalSet::new();
        let _guard = set.enter();

        loop {
            // 1. Run lua threads until exit or there are none left,
            // if any thread was resumed it may have spawned futures
            let resumed_lua = self.run_lua_threads();

            // 2. If we got a manual exit code from lua we should
            // not try to wait for any pending futures to complete
            if self.state.has_exit_code() {
                break;
            }

            // 3. Keep resuming futures until we get a new lua thread to
            // resume, or until we don't have any futures left to wait for
            let resumed_fut = self.run_futures().await;

            // 4. If we did not resume any lua threads, and we have no futures
            // remaining either, we have now run the scheduler until completion
            if !resumed_lua && !resumed_fut {
                break;
            }
        }

        if let Some(code) = self.state.exit_code() {
            ExitCode::from(code)
        } else if self.state.has_errored() {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    }

    /**
        Runs a script with the given `script_name` and `script_contents` to completion.

        Refer to [`run_to_completion`] for additional details.
    */
    pub async fn run_main(
        self,
        script_name: impl AsRef<str>,
        script_contents: impl AsRef<[u8]>,
    ) -> ExitCode {
        let main_fn = self
            .lua
            .load(script_contents.as_ref())
            .set_name(script_name.as_ref())
            .into_function()
            .expect("Failed to create function for main");

        let main_thread = main_fn
            .into_owned_lua_thread(&self.lua)
            .expect("Failed to create thread for main");

        self.push_back(main_thread, EMPTY_MULTI_VALUE)
            .expect("Failed to enqueue thread for main");

        self.run_to_completion().await
    }
}
