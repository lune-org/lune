use std::{process::ExitCode, sync::Arc};

use futures_util::StreamExt;
use mlua::prelude::*;

use tokio::task::LocalSet;

use super::Scheduler;

impl<'lua, 'fut> Scheduler<'lua, 'fut>
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

        // Pop threads from the scheduler until there are none left
        while let Some(thread) = self
            .pop_thread()
            .expect("Failed to pop thread from scheduler")
        {
            // Deconstruct the scheduler thread into its parts
            let thread_id = thread.id();
            let (thread, args) = thread.into_inner(self.lua);

            // Resume the thread, ensuring that the schedulers
            // current thread id is set correctly for error catching
            self.state.set_current_thread_id(Some(thread_id));
            let res = thread.resume::<_, LuaMultiValue>(args);
            self.state.set_current_thread_id(None);

            resumed_any = true;

            // If we got any resumption (lua-side) error, increment
            // the error count of the scheduler so we can exit with
            // a non-zero exit code, and print it out to stderr
            // TODO: Pretty print the lua error here
            if let Err(err) = &res {
                self.state.increment_error_count();
                eprint!("{err}");
            }

            // Send results of resuming this thread to any listeners
            if let Some(sender) = self.thread_senders.borrow_mut().remove(&thread_id) {
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
}
