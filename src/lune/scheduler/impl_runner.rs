use std::{process::ExitCode, sync::Arc};

use futures_util::StreamExt;
use mlua::prelude::*;

use tokio::task::LocalSet;
use tracing::debug;

use crate::lune::util::traits::LuaEmitErrorExt;

use super::Scheduler;

impl<'fut> Scheduler<'fut> {
    /**
        Runs all lua threads to completion.
    */
    fn run_lua_threads(&self, lua: &Lua) {
        if self.state.has_exit_code() {
            return;
        }

        let mut count = 0;

        // Pop threads from the scheduler until there are none left
        while let Some(thread) = self
            .pop_thread()
            .expect("Failed to pop thread from scheduler")
        {
            // Deconstruct the scheduler thread into its parts
            let thread_id = thread.id();
            let (thread, args) = thread.into_inner(lua);

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

            count += 1;

            // If we got any resumption (lua-side) error, increment
            // the error count of the scheduler so we can exit with
            // a non-zero exit code, and print it out to stderr
            if let Err(err) = &res {
                self.state.increment_error_count();
                lua.emit_error(err.clone());
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
                            Ok(v) => Ok(Arc::new(lua.create_registry_value(v.into_vec()).expect(
                                "Failed to store thread results in registry - out of memory",
                            ))),
                        };
                        sender
                            .send(stored)
                            .expect("Failed to broadcast thread results");
                    }
                }
            }

            if self.state.has_exit_code() {
                break;
            }
        }

        if count > 0 {
            debug! {
                %count,
                "resumed lua"
            }
        }
    }

    /**
        Runs the next lua future to completion.

        Panics if no lua future is queued.
    */
    async fn run_future_lua(&self) {
        let mut futs = self
            .futures_lua
            .try_lock()
            .expect("Failed to lock lua futures for resumption");
        assert!(futs.len() > 0, "No lua futures are queued");
        futs.next().await;
    }

    /**
        Runs the next background future to completion.

        Panics if no background future is queued.
    */
    async fn run_future_background(&self) {
        let mut futs = self
            .futures_background
            .try_lock()
            .expect("Failed to lock background futures for resumption");
        assert!(futs.len() > 0, "No background futures are queued");
        futs.next().await;
    }

    /**
        Runs as many futures as possible, until a new lua thread
        is ready, or an exit code has been set for the scheduler.

        ### Implementation details

        Running futures on our scheduler consists of a couple moving parts:

        1. An unordered futures queue for lua (main thread, local) futures
        2. An unordered futures queue for background (multithreaded, 'static lifetime) futures
        3. A signal for breaking out of futures resumption

        The two unordered futures queues need to run concurrently,
        but since `FuturesUnordered` returns instantly if it does
        not currently have any futures queued on it, we need to do
        this branching loop, checking if each queue has futures first.

        We also need to listen for our signal, to see if we should break out of resumption:

        * Always break out of resumption if a new lua thread is ready
        * Always break out of resumption if an exit code has been set
        * Break out of lua futures resumption if we have a new background future
        * Break out of background futures resumption if we have a new lua future

        We need to listen for both future queues concurrently,
        and break out whenever the other corresponding queue has
        a new future, since the other queue may resume sooner.
    */
    async fn run_futures(&self) {
        let (mut has_lua, mut has_background) = self.has_futures();
        if !has_lua && !has_background {
            return;
        }

        let mut rx = self.futures_signal.subscribe();
        let mut count = 0;
        while has_lua || has_background {
            if has_lua && has_background {
                tokio::select! {
                    _ = self.run_future_lua() => {},
                    _ = self.run_future_background() => {},
                    msg = rx.recv() => {
                        if let Ok(msg) = msg {
                            if msg.should_break_futures() {
                                break;
                            }
                        }
                    }
                }
                count += 1;
            } else if has_lua {
                tokio::select! {
                    _ = self.run_future_lua() => {},
                    msg = rx.recv() => {
                        if let Ok(msg) = msg {
                            if msg.should_break_lua_futures() {
                                break;
                            }
                        }
                    }
                }
                count += 1;
            } else if has_background {
                tokio::select! {
                    _ = self.run_future_background() => {},
                    msg = rx.recv() => {
                        if let Ok(msg) = msg {
                            if msg.should_break_background_futures() {
                                break;
                            }
                        }
                    }
                }
                count += 1;
            }
            (has_lua, has_background) = self.has_futures();
        }

        if count > 0 {
            debug! {
                %count,
                "resumed lua futures"
            }
        }
    }

    /**
        Runs the scheduler to completion in a [`LocalSet`],
        both normal lua threads and futures, prioritizing
        lua threads over completion of any pending futures.

        Will emit lua output and errors to stdout and stderr.
    */
    pub async fn run_to_completion(&self, lua: &Lua) -> ExitCode {
        if let Some(code) = self.state.exit_code() {
            return ExitCode::from(code);
        }

        let set = LocalSet::new();
        let _guard = set.enter();

        loop {
            // 1. Run lua threads until exit or there are none left
            self.run_lua_threads(lua);

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
            debug! {
                %code,
                "scheduler ran to completion"
            };
            ExitCode::from(code)
        } else if self.state.has_errored() {
            debug!("scheduler ran to completion, with failure");
            ExitCode::FAILURE
        } else {
            debug!("scheduler ran to completion, with success");
            ExitCode::SUCCESS
        }
    }
}
