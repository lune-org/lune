use std::{process::ExitCode, sync::Arc};

use futures_util::StreamExt;
use mlua::prelude::*;

use tokio::task::LocalSet;

use crate::lune::util::traits::LuaEmitErrorExt;

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

            // Make sure this thread is still resumable, it might have
            // been resumed somewhere else or even have been cancelled
            if thread.status() != LuaThreadStatus::Resumable {
                continue;
            }

            // Resume the thread, ensuring that the schedulers
            // current thread id is set correctly for error catching
            self.state.set_current_thread_id(Some(thread_id));
            let res = thread.resume::<_, LuaMultiValue>(args);
            self.state.set_current_thread_id(None);

            resumed_any = true;

            // If we got any resumption (lua-side) error, increment
            // the error count of the scheduler so we can exit with
            // a non-zero exit code, and print it out to stderr
            if let Err(err) = &res {
                self.state.increment_error_count();
                self.lua.emit_error(err.clone());
            }

            // If the thread has finished running completely,
            // send results of final resume to any listeners
            if thread.status() != LuaThreadStatus::Resumable {
                // NOTE: Threads that were spawned to resume
                // with an error will not have a result sender
                if let Some(sender) = self.thread_senders.borrow_mut().remove(&thread_id) {
                    if sender.receiver_count() > 0 {
                        let stored = match res {
                            Err(e) => Err(e),
                            Ok(v) => Ok(Arc::new(
                                self.lua
                                    .create_registry_value(v.into_vec())
                                    .expect("Failed to store return values in registry"),
                            )),
                        };
                        sender
                            .send(stored)
                            .expect("Failed to broadcast return values of thread");
                    }
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
    */
    async fn run_futures_lua(&self) {
        let mut futs = self
            .futures_lua
            .try_lock()
            .expect("Failed to lock lua futures for resumption");

        while futs.next().await.is_some() {
            if self.has_thread() {
                break;
            }
        }
    }

    /**
        Runs background futures until none are left or a future spawned a new lua thread.
    */
    async fn run_futures_background(&self) {
        let mut futs = self
            .futures_background
            .try_lock()
            .expect("Failed to lock background futures for resumption");

        while futs.next().await.is_some() {
            if self.has_thread() {
                break;
            }
        }
    }

    async fn run_futures(&self) {
        let mut rx = self.futures_break_signal.subscribe();

        tokio::select! {
            _ = self.run_futures_lua() => {},
            _ = self.run_futures_background() => {},
            _ = rx.recv() => {},
        };
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
            // 1. Run lua threads until exit or there are none left
            self.run_lua_threads();

            // 2. If we got a manual exit code from lua we should
            // not try to wait for any pending futures to complete
            if self.state.has_exit_code() {
                break;
            }

            // 3. Keep resuming futures until there are no futures left to
            // resume, or until we manually break out of resumption for any
            // reason, this may be because a future spawned a new lua thread
            self.run_futures().await;

            // 4. Once again, check for an exit code, in case a future sets one
            if self.state.has_exit_code() {
                break;
            }

            // 5. If we have no lua threads or futures remaining,
            // we have now run the scheduler until completion
            let (has_future_lua, has_future_background) = self.has_futures();
            if !has_future_lua && !has_future_background && !self.has_thread() {
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
