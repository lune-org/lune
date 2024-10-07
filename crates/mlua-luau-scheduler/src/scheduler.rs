#![allow(clippy::module_name_repetitions)]

use std::{
    cell::Cell,
    process::ExitCode,
    rc::{Rc, Weak as WeakRc},
    sync::{Arc, Weak as WeakArc},
    thread::panicking,
};

use futures_lite::prelude::*;
use mlua::prelude::*;

use async_executor::{Executor, LocalExecutor};
use tracing::{debug, instrument, trace, trace_span};

use crate::{
    error_callback::ThreadErrorCallback,
    exit::Exit,
    queue::{DeferredThreadQueue, FuturesQueue, SpawnedThreadQueue},
    result_map::ThreadResultMap,
    status::Status,
    thread_id::ThreadId,
    traits::IntoLuaThread,
    util::{run_until_yield, ThreadResult},
};

const ERR_METADATA_ALREADY_ATTACHED: &str = "\
Lua state already has scheduler metadata attached!\
\nThis may be caused by running multiple schedulers on the same Lua state, or a call to Scheduler::run being cancelled.\
\nOnly one scheduler can be used per Lua state at once, and schedulers must always run until completion.\
";

const ERR_METADATA_REMOVED: &str = "\
Lua state scheduler metadata was unexpectedly removed!\
\nThis should never happen, and is likely a bug in the scheduler.\
";

const ERR_SET_CALLBACK_WHEN_RUNNING: &str = "\
Cannot set error callback when scheduler is running!\
";

/**
    A scheduler for running Lua threads and async tasks.
*/
#[derive(Clone)]
pub struct Scheduler<'lua> {
    lua: &'lua Lua,
    queue_spawn: SpawnedThreadQueue,
    queue_defer: DeferredThreadQueue,
    error_callback: ThreadErrorCallback,
    result_map: ThreadResultMap,
    status: Rc<Cell<Status>>,
    exit: Exit,
}

impl<'lua> Scheduler<'lua> {
    /**
        Creates a new scheduler for the given Lua state.

        This scheduler will have a default error callback that prints errors to stderr.

        # Panics

        Panics if the given Lua state already has a scheduler attached to it.
    */
    #[must_use]
    pub fn new(lua: &'lua Lua) -> Scheduler<'lua> {
        let queue_spawn = SpawnedThreadQueue::new();
        let queue_defer = DeferredThreadQueue::new();
        let error_callback = ThreadErrorCallback::default();
        let result_map = ThreadResultMap::new();
        let exit = Exit::new();

        assert!(
            lua.app_data_ref::<SpawnedThreadQueue>().is_none(),
            "{ERR_METADATA_ALREADY_ATTACHED}"
        );
        assert!(
            lua.app_data_ref::<DeferredThreadQueue>().is_none(),
            "{ERR_METADATA_ALREADY_ATTACHED}"
        );
        assert!(
            lua.app_data_ref::<ThreadErrorCallback>().is_none(),
            "{ERR_METADATA_ALREADY_ATTACHED}"
        );
        assert!(
            lua.app_data_ref::<ThreadResultMap>().is_none(),
            "{ERR_METADATA_ALREADY_ATTACHED}"
        );
        assert!(
            lua.app_data_ref::<Exit>().is_none(),
            "{ERR_METADATA_ALREADY_ATTACHED}"
        );

        lua.set_app_data(queue_spawn.clone());
        lua.set_app_data(queue_defer.clone());
        lua.set_app_data(error_callback.clone());
        lua.set_app_data(result_map.clone());
        lua.set_app_data(exit.clone());

        let status = Rc::new(Cell::new(Status::NotStarted));

        Scheduler {
            lua,
            queue_spawn,
            queue_defer,
            error_callback,
            result_map,
            status,
            exit,
        }
    }

    /**
        Sets the current status of this scheduler and emits relevant tracing events.
    */
    fn set_status(&self, status: Status) {
        debug!(status = ?status, "status");
        self.status.set(status);
    }

    /**
        Returns the current status of this scheduler.
    */
    #[must_use]
    pub fn status(&self) -> Status {
        self.status.get()
    }

    /**
        Sets the error callback for this scheduler.

        This callback will be called whenever a Lua thread errors.

        Overwrites any previous error callback.

        # Panics

        Panics if the scheduler is currently running.
    */
    pub fn set_error_callback(&self, callback: impl Fn(LuaError) + Send + 'static) {
        assert!(
            !self.status().is_running(),
            "{ERR_SET_CALLBACK_WHEN_RUNNING}"
        );
        self.error_callback.replace(callback);
    }

    /**
        Clears the error callback for this scheduler.

        This will remove any current error callback, including default(s).

        # Panics

        Panics if the scheduler is currently running.
    */
    pub fn remove_error_callback(&self) {
        assert!(
            !self.status().is_running(),
            "{ERR_SET_CALLBACK_WHEN_RUNNING}"
        );
        self.error_callback.clear();
    }

    /**
        Gets the exit code for this scheduler, if one has been set.
    */
    #[must_use]
    pub fn get_exit_code(&self) -> Option<ExitCode> {
        self.exit.get()
    }

    /**
        Sets the exit code for this scheduler.

        This will cause [`Scheduler::run`] to exit immediately.
    */
    pub fn set_exit_code(&self, code: ExitCode) {
        self.exit.set(code);
    }

    /**
        Spawns a chunk / function / thread onto the scheduler queue.

        Threads are guaranteed to be resumed in the order that they were pushed to the queue.

        # Returns

        Returns a [`ThreadId`] that can be used to retrieve the result of the thread.

        Note that the result may not be available until [`Scheduler::run`] completes.

        # Errors

        Errors when out of memory.
    */
    pub fn push_thread_front(
        &self,
        thread: impl IntoLuaThread<'lua>,
        args: impl IntoLuaMulti<'lua>,
    ) -> LuaResult<ThreadId> {
        let id = self.queue_spawn.push_item(self.lua, thread, args)?;
        self.result_map.track(id);
        Ok(id)
    }

    /**
        Defers a chunk / function / thread onto the scheduler queue.

        Deferred threads are guaranteed to run after all spawned threads either yield or complete.

        Threads are guaranteed to be resumed in the order that they were pushed to the queue.

        # Returns

        Returns a [`ThreadId`] that can be used to retrieve the result of the thread.

        Note that the result may not be available until [`Scheduler::run`] completes.

        # Errors

        Errors when out of memory.
    */
    pub fn push_thread_back(
        &self,
        thread: impl IntoLuaThread<'lua>,
        args: impl IntoLuaMulti<'lua>,
    ) -> LuaResult<ThreadId> {
        let id = self.queue_defer.push_item(self.lua, thread, args)?;
        self.result_map.track(id);
        Ok(id)
    }

    /**
        Gets the tracked result for the [`LuaThread`] with the given [`ThreadId`].

        Depending on the current [`Scheduler::status`], this method will return:

        - [`Status::NotStarted`]: returns `None`.
        - [`Status::Running`]: may return `Some(Ok(v))` or `Some(Err(e))`, but it is not guaranteed.
        - [`Status::Completed`]: returns `Some(Ok(v))` or `Some(Err(e))`.

        Note that this method also takes the value out of the scheduler and
        stops tracking the given thread, so it may only be called once.

        Any subsequent calls after this method returns `Some` will return `None`.
    */
    #[must_use]
    pub fn get_thread_result(&self, id: ThreadId) -> Option<LuaResult<LuaMultiValue<'lua>>> {
        self.result_map.remove(id).map(|r| r.value(self.lua))
    }

    /**
        Waits for the [`LuaThread`] with the given [`ThreadId`] to complete.

        This will return instantly if the thread has already completed.
    */
    pub async fn wait_for_thread(&self, id: ThreadId) {
        self.result_map.listen(id).await;
    }

    /**
        Runs the scheduler until all Lua threads have completed.

        Note that the given Lua state must be the same one that was
        used to create this scheduler, otherwise this method will panic.

        # Panics

        Panics if the given Lua state already has a scheduler attached to it.
    */
    #[allow(clippy::too_many_lines)]
    #[instrument(level = "debug", name = "Scheduler::run", skip(self))]
    pub async fn run(&self) {
        /*
            Create new executors to use - note that we do not need create multiple executors
            for work stealing, the user may do that themselves if they want to and it will work
            just fine, as long as anything async is .await-ed from within a Lua async function.

            The main purpose of the two executors here is just to have one with
            the Send bound, and another (local) one without it, for Lua scheduling.

            We also use the main executor to drive the main loop below forward,
            saving a tiny bit of processing from going on the Lua executor itself.
        */
        let local_exec = LocalExecutor::new();
        let main_exec = Arc::new(Executor::new());
        let fut_queue = Rc::new(FuturesQueue::new());

        /*
            Store the main executor and queue in Lua, so that they may be used with LuaSchedulerExt.

            Also ensure we do not already have an executor or queues - these are definite user errors
            and may happen if the user tries to run multiple schedulers on the same Lua state at once.
        */
        assert!(
            self.lua.app_data_ref::<WeakArc<Executor>>().is_none(),
            "{ERR_METADATA_ALREADY_ATTACHED}"
        );
        assert!(
            self.lua.app_data_ref::<WeakRc<FuturesQueue>>().is_none(),
            "{ERR_METADATA_ALREADY_ATTACHED}"
        );

        self.lua.set_app_data(Arc::downgrade(&main_exec));
        self.lua.set_app_data(Rc::downgrade(&fut_queue.clone()));

        /*
            Manually tick the Lua executor, while running under the main executor.
            Each tick we wait for the next action to perform, in prioritized order:

            1. The exit event is triggered by setting an exit code
            2. A Lua thread is available to run on the spawned queue
            3. A Lua thread is available to run on the deferred queue
            4. A new thread-local future is available to run on the local executor
            5. Task(s) scheduled on the Lua executor have made progress and should be polled again

            This ordering is vital to ensure that we don't accidentally exit the main loop
            when there are new Lua threads to enqueue and potentially more work to be done.
        */
        let fut = async {
            let result_map = self.result_map.clone();
            let process_thread = |thread: LuaThread<'lua>, args| {
                // NOTE: Thread may have been cancelled from Lua
                // before we got here, so we need to check it again
                if thread.status() == LuaThreadStatus::Resumable {
                    // Check if we should be tracking this thread
                    let id = ThreadId::from(&thread);
                    let id_tracked = result_map.is_tracked(id);
                    let result_map_inner = if id_tracked {
                        Some(result_map.clone())
                    } else {
                        None
                    };
                    // Create our future which will run the thread and store its final result
                    let fut = async move {
                        if id_tracked {
                            // Run until yield and check if we got a final result
                            if let Some(res) = run_until_yield(thread.clone(), args).await {
                                if let Err(e) = res.as_ref() {
                                    self.error_callback.call(e);
                                }
                                if thread.status() != LuaThreadStatus::Resumable {
                                    let thread_res = ThreadResult::new(res, self.lua);
                                    result_map_inner.unwrap().insert(id, thread_res);
                                }
                            }
                        } else {
                            // Just run until yield
                            if let Some(res) = run_until_yield(thread, args).await {
                                if let Err(e) = res.as_ref() {
                                    self.error_callback.call(e);
                                }
                            }
                        }
                    };
                    // Spawn it on the executor
                    local_exec.spawn(fut).detach();
                }
            };

            loop {
                let fut_exit = self.exit.listen(); // 1
                let fut_spawn = self.queue_spawn.wait_for_item(); // 2
                let fut_defer = self.queue_defer.wait_for_item(); // 3
                let fut_futs = fut_queue.wait_for_item(); // 4

                local_exec
                    .run(
                        fut_exit
                            .race(fut_spawn)
                            .race(fut_defer)
                            .race(fut_futs)
                            .race(local_exec.tick()),
                    )
                    .await;

                // Check if we should exit
                if self.exit.get().is_some() {
                    debug!("exit signal received");
                    break;
                }

                // Process spawned threads first, then deferred threads, then futures
                let mut num_spawned = 0;
                let mut num_deferred = 0;
                let mut num_futures = 0;
                {
                    let _span = trace_span!("Scheduler::drain_spawned").entered();
                    for (thread, args) in self.queue_spawn.drain_items(self.lua) {
                        process_thread(thread, args);
                        num_spawned += 1;
                    }
                }
                {
                    let _span = trace_span!("Scheduler::drain_deferred").entered();
                    for (thread, args) in self.queue_defer.drain_items(self.lua) {
                        process_thread(thread, args);
                        num_deferred += 1;
                    }
                }
                {
                    let _span = trace_span!("Scheduler::drain_futures").entered();
                    for fut in fut_queue.drain_items() {
                        local_exec.spawn(fut).detach();
                        num_futures += 1;
                    }
                }

                // Empty executor = we didn't spawn any new Lua tasks
                // above, and there are no remaining tasks to run later
                let completed = local_exec.is_empty()
                    && self.queue_spawn.is_empty()
                    && self.queue_defer.is_empty();

                trace!(
                    futures_spawned = num_futures,
                    // futures_processed = num_processed,
                    lua_threads_spawned = num_spawned,
                    lua_threads_deferred = num_deferred,
                    "loop"
                );

                if completed {
                    break;
                }
            }
        };

        // Run the executor inside a span until all lua threads complete
        self.set_status(Status::Running);
        main_exec.run(fut).await;
        self.set_status(Status::Completed);

        // Clean up
        self.lua
            .remove_app_data::<WeakArc<Executor>>()
            .expect(ERR_METADATA_REMOVED);
        self.lua
            .remove_app_data::<WeakRc<FuturesQueue>>()
            .expect(ERR_METADATA_REMOVED);
    }
}

impl Drop for Scheduler<'_> {
    fn drop(&mut self) {
        if panicking() {
            // Do not cause further panics if already panicking, as
            // this may abort the program instead of safely unwinding
            self.lua.remove_app_data::<SpawnedThreadQueue>();
            self.lua.remove_app_data::<DeferredThreadQueue>();
            self.lua.remove_app_data::<ThreadErrorCallback>();
            self.lua.remove_app_data::<ThreadResultMap>();
            self.lua.remove_app_data::<Exit>();
        } else {
            // In any other case we panic if metadata was removed incorrectly
            self.lua
                .remove_app_data::<SpawnedThreadQueue>()
                .expect(ERR_METADATA_REMOVED);
            self.lua
                .remove_app_data::<DeferredThreadQueue>()
                .expect(ERR_METADATA_REMOVED);
            self.lua
                .remove_app_data::<ThreadErrorCallback>()
                .expect(ERR_METADATA_REMOVED);
            self.lua
                .remove_app_data::<ThreadResultMap>()
                .expect(ERR_METADATA_REMOVED);
            self.lua
                .remove_app_data::<Exit>()
                .expect(ERR_METADATA_REMOVED);
        }
    }
}
