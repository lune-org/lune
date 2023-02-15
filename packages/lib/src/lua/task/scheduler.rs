use core::panic;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    process::ExitCode,
    time::Duration,
};

use futures_util::{future::LocalBoxFuture, stream::FuturesUnordered, Future, StreamExt};
use mlua::prelude::*;

use tokio::{
    sync::{mpsc, Mutex as AsyncMutex},
    time::{sleep, Instant},
};

use crate::utils::table::TableBuilder;

use super::{
    async_handle::TaskSchedulerAsyncHandle, message::TaskSchedulerMessage,
    result::TaskSchedulerState, task_kind::TaskKind, task_reference::TaskReference,
};

type TaskFutureRets<'fut> = LuaResult<Option<LuaMultiValue<'fut>>>;
type TaskFuture<'fut> = LocalBoxFuture<'fut, (TaskReference, TaskFutureRets<'fut>)>;

const TASK_ASYNC_IMPL_LUA: &str = r#"
resumeAsync(thread(), ...)
return yield()
"#;

/// A struct representing a task contained in the task scheduler
#[derive(Debug)]
pub struct Task {
    thread: LuaRegistryKey,
    args: LuaRegistryKey,
    queued_at: Instant,
}

/// A task scheduler that implements task queues
/// with instant, deferred, and delayed tasks
#[derive(Debug)]
pub struct TaskScheduler<'fut> {
    /*
        Lots of cell and refcell here, however we need full interior mutability and never outer
        since the scheduler struct may be accessed from lua more than once at the same time.

        An example of this is the implementation of coroutine.resume, which instantly resumes the given
        task, where the task getting resumed may also create new scheduler tasks during its resumption.

        The same goes for values used during resumption of futures (`futures` and `futures_rx`)
        which must use async-aware mutexes to be cancellation safe across await points.
    */
    // Internal state & flags
    lua: &'static Lua,
    guid: Cell<usize>,
    guid_running: Cell<Option<usize>>,
    pub(super) exit_code: Cell<Option<ExitCode>>,
    // Blocking tasks
    pub(super) tasks: RefCell<HashMap<TaskReference, Task>>,
    pub(super) tasks_queue_blocking: RefCell<VecDeque<TaskReference>>,
    // Future tasks & objects for waking
    pub(super) futures: AsyncMutex<FuturesUnordered<TaskFuture<'fut>>>,
    pub(super) futures_registered_count: Cell<usize>,
    futures_tx: mpsc::UnboundedSender<TaskSchedulerMessage>,
    futures_rx: AsyncMutex<mpsc::UnboundedReceiver<TaskSchedulerMessage>>,
}

impl<'fut> TaskScheduler<'fut> {
    /**
        Creates a new task scheduler.
    */
    pub fn new(lua: &'static Lua) -> LuaResult<Self> {
        let (tx, rx) = mpsc::unbounded_channel();
        Ok(Self {
            lua,
            guid: Cell::new(0),
            guid_running: Cell::new(None),
            exit_code: Cell::new(None),
            tasks: RefCell::new(HashMap::new()),
            tasks_queue_blocking: RefCell::new(VecDeque::new()),
            futures: AsyncMutex::new(FuturesUnordered::new()),
            futures_tx: tx,
            futures_rx: AsyncMutex::new(rx),
            futures_registered_count: Cell::new(0),
        })
    }

    /**
        Consumes and leaks the task scheduler,
        returning a static reference `&'static TaskScheduler`.

        This function is useful when the task scheduler object is
        supposed to live for the remainder of the program's life.

        Note that dropping the returned reference will cause a memory leak.
    */
    pub fn into_static(self) -> &'static Self {
        Box::leak(Box::new(self))
    }

    /**
        Stores the exit code for the task scheduler.

        This will be passed back to the Rust thread that is running the task scheduler,
        in the [`TaskSchedulerState`] returned on resumption of the task scheduler queue.

        Setting this exit code will signal to that thread that it
        should stop resuming tasks, and gracefully terminate the program.
    */
    pub fn set_exit_code(&self, code: ExitCode) {
        self.exit_code.set(Some(code));
    }

    /**
        Checks if a task still exists in the scheduler.

        A task may no longer exist in the scheduler if it has been manually
        cancelled and removed by calling [`TaskScheduler::cancel_task()`].
    */
    #[allow(dead_code)]
    pub fn contains_task(&self, reference: TaskReference) -> bool {
        self.tasks.borrow().contains_key(&reference)
    }

    /**
        Creates a new task, storing a new Lua thread
        for it, as well as the arguments to give the
        thread on resumption, in the Lua registry.

        Note that this task will ***not*** resume on its
        own, it needs to be used together with either the
        scheduling functions or [`TaskScheduler::resume_task`].
    */
    pub fn create_task(
        &self,
        kind: TaskKind,
        thread_or_function: LuaValue<'_>,
        thread_args: Option<LuaMultiValue<'_>>,
        guid_to_reuse: Option<usize>,
    ) -> LuaResult<TaskReference> {
        // Get or create a thread from the given argument
        let task_thread = match thread_or_function {
            LuaValue::Thread(t) => t,
            LuaValue::Function(f) => self.lua.create_thread(f)?,
            value => {
                return Err(LuaError::RuntimeError(format!(
                    "Argument must be a thread or function, got {}",
                    value.type_name()
                )))
            }
        };
        // Store the thread and its arguments in the registry
        // NOTE: We must convert to a vec since multis
        // can't be stored in the registry directly
        let task_args_vec: Option<Vec<LuaValue>> = thread_args.map(|opt| opt.into_vec());
        let task_args_key: LuaRegistryKey = self.lua.create_registry_value(task_args_vec)?;
        let task_thread_key: LuaRegistryKey = self.lua.create_registry_value(task_thread)?;
        // Create the full task struct
        let queued_at = Instant::now();
        let task = Task {
            thread: task_thread_key,
            args: task_args_key,
            queued_at,
        };
        // Create the task ref to use
        let task_ref = if let Some(reusable_guid) = guid_to_reuse {
            TaskReference::new(kind, reusable_guid)
        } else {
            let guid = self.guid.get();
            self.guid.set(guid + 1);
            TaskReference::new(kind, guid)
        };
        // Add the task to the scheduler
        {
            let mut tasks = self.tasks.borrow_mut();
            tasks.insert(task_ref, task);
        }
        Ok(task_ref)
    }

    /**
        Cancels a task, if the task still exists in the scheduler.

        It is possible to hold one or more task references that point
        to a task that no longer exists in the scheduler, and calling
        this method with one of those references will return `false`.
    */
    pub fn remove_task(&self, reference: TaskReference) -> LuaResult<bool> {
        /*
            Remove the task from the task list and the Lua registry

            This is all we need to do since resume_task will always
            ignore resumption of any task that no longer exists there

            This does lead to having some amount of "junk" futures that will
            build up in the queue but these will get cleaned up and not block
            the program from exiting since the scheduler only runs until there
            are no tasks left in the task list, the futures do not matter there
        */
        let mut found = false;
        let mut tasks = self.tasks.borrow_mut();
        // Unfortunately we have to loop through to find which task
        // references to remove instead of removing directly since
        // tasks can switch kinds between instant, deferred, future
        let tasks_to_remove: Vec<_> = tasks
            .keys()
            .filter(|task_ref| task_ref.id() == reference.id())
            .copied()
            .collect();
        for task_ref in tasks_to_remove {
            if let Some(task) = tasks.remove(&task_ref) {
                self.lua.remove_registry_value(task.thread)?;
                self.lua.remove_registry_value(task.args)?;
                found = true;
            }
        }
        Ok(found)
    }

    /**
        Resumes a task, if the task still exists in the scheduler.

        A task may no longer exist in the scheduler if it has been manually
        cancelled and removed by calling [`TaskScheduler::cancel_task()`].

        This will be a no-op if the task no longer exists.
    */
    pub fn resume_task<'a>(
        &self,
        reference: TaskReference,
        override_args: Option<LuaResult<LuaMultiValue<'a>>>,
    ) -> LuaResult<LuaMultiValue<'a>> {
        let task = {
            let mut tasks = self.tasks.borrow_mut();
            match tasks.remove(&reference) {
                Some(task) => task,
                None => return Ok(LuaMultiValue::new()), // Task was removed
            }
        };
        let thread: LuaThread = self.lua.registry_value(&task.thread)?;
        let args_opt_res = override_args.or_else(|| {
            Ok(self
                .lua
                .registry_value::<Option<Vec<LuaValue>>>(&task.args)
                .expect("Failed to get stored args for task")
                .map(LuaMultiValue::from_vec))
            .transpose()
        });
        self.lua.remove_registry_value(task.thread)?;
        self.lua.remove_registry_value(task.args)?;
        if let Some(args_res) = args_opt_res {
            match args_res {
                Err(e) => Err(e), // FIXME: We need to throw this error in lua to let pcall & friends handle it properly
                Ok(args) => {
                    self.guid_running.set(Some(reference.id()));
                    let rets = thread.resume::<_, LuaMultiValue>(args);
                    self.guid_running.set(None);
                    rets
                }
            }
        } else {
            /*
                The tasks did not get any arguments from either:

                - Providing arguments at the call site for creating the task
                - Returning arguments from a future that created this task

                The only tasks that do not get any arguments from either
                of those sources are waiting tasks, and waiting tasks
                want the amount of time waited returned to them.
            */
            let elapsed = task.queued_at.elapsed().as_secs_f64();
            self.guid_running.set(Some(reference.id()));
            let rets = thread.resume::<_, LuaMultiValue>(elapsed);
            self.guid_running.set(None);
            rets
        }
    }

    /**
        Queues a new task to run on the task scheduler.

        When we want to schedule a task to resume instantly after the
        currently running task we should pass `after_current_resume = true`.

        This is useful in cases such as our task.spawn implementation:

        ```lua
        task.spawn(function()
            -- This will be a new task, but it should
            -- also run right away, until the first yield
        end)
        -- Here we have either yielded or finished the above task
        ```
    */
    pub(super) fn queue_blocking_task(
        &self,
        kind: TaskKind,
        thread_or_function: LuaValue<'_>,
        thread_args: Option<LuaMultiValue<'_>>,
        guid_to_reuse: Option<usize>,
    ) -> LuaResult<TaskReference> {
        if kind == TaskKind::Future {
            panic!("Tried to schedule future using normal task schedule method")
        }
        let task_ref = self.create_task(kind, thread_or_function, thread_args, guid_to_reuse)?;
        // Add the task to the front of the queue, unless it
        // should be deferred, in that case add it to the back
        let mut queue = self.tasks_queue_blocking.borrow_mut();
        let num_prev_blocking_tasks = queue.len();
        if kind == TaskKind::Deferred {
            queue.push_back(task_ref);
        } else {
            queue.push_front(task_ref);
        }
        /*
            If we had any previous task and are currently async
            waiting on tasks, we should send a signal to wake up
            and run the new blocking task that was just queued

            This can happen in cases such as an async http
            server waking up from a connection and then wanting to
            run a lua callback in response, to create the.. response
        */
        if num_prev_blocking_tasks == 0 {
            self.futures_tx
                .send(TaskSchedulerMessage::NewBlockingTaskReady)
                .expect("Futures waker channel was closed")
        }
        Ok(task_ref)
    }

    /**
        Queues a new future to run on the task scheduler.
    */
    pub(super) fn queue_async_task(
        &self,
        thread_or_function: LuaValue<'_>,
        thread_args: Option<LuaMultiValue<'_>>,
        guid_to_reuse: Option<usize>,
        fut: impl Future<Output = TaskFutureRets<'fut>> + 'fut,
    ) -> LuaResult<TaskReference> {
        let task_ref = self.create_task(
            TaskKind::Future,
            thread_or_function,
            thread_args,
            guid_to_reuse,
        )?;
        let futs = self
            .futures
            .try_lock()
            .expect("Failed to get lock on futures");
        futs.push(Box::pin(async move {
            let result = fut.await;
            (task_ref, result)
        }));
        Ok(task_ref)
    }

    /**
        Schedules a lua thread or function to resume ***first*** during this
        resumption point, ***skipping ahead*** of any other currently queued tasks.

        The given lua thread or function will be resumed
        using the given `thread_args` as its argument(s).
    */
    pub fn schedule_next(
        &self,
        thread_or_function: LuaValue<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference> {
        self.queue_blocking_task(
            TaskKind::Instant,
            thread_or_function,
            Some(thread_args),
            None,
        )
    }

    /**
        Schedules a lua thread or function to resume ***after all***
        currently resuming tasks, during this resumption point.

        The given lua thread or function will be resumed
        using the given `thread_args` as its argument(s).
    */
    pub fn schedule_deferred(
        &self,
        thread_or_function: LuaValue<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference> {
        self.queue_blocking_task(
            TaskKind::Deferred,
            thread_or_function,
            Some(thread_args),
            None,
        )
    }

    /**
        Schedules a lua thread or function to
        be resumed after waiting asynchronously.

        The given lua thread or function will be resumed
        using the given `thread_args` as its argument(s).
    */
    pub fn schedule_delayed(
        &self,
        after_secs: f64,
        thread_or_function: LuaValue<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference> {
        self.queue_async_task(thread_or_function, Some(thread_args), None, async move {
            sleep(Duration::from_secs_f64(after_secs)).await;
            Ok(None)
        })
    }

    /**
        Schedules a lua thread or function to
        be resumed after waiting asynchronously.

        The given lua thread or function will be resumed
        using the elapsed time as its one and only argument.
    */
    pub fn schedule_wait(
        &self,
        after_secs: f64,
        thread_or_function: LuaValue<'_>,
    ) -> LuaResult<TaskReference> {
        self.queue_async_task(
            thread_or_function,
            None,
            // Wait should recycle the guid of the current task,
            // which ensures that the TaskReference is identical and
            // that any waits inside of spawned tasks will also cancel
            self.guid_running.get(),
            async move {
                sleep(Duration::from_secs_f64(after_secs)).await;
                Ok(None)
            },
        )
    }

    /**
        Schedules a lua thread or function
        to be resumed after running a future.

        The given lua thread or function will be resumed
        using the optional arguments returned by the future.
    */
    #[allow(dead_code)]
    pub fn schedule_async<'sched, R, F, FR>(
        &'sched self,
        thread_or_function: LuaValue<'_>,
        func: F,
    ) -> LuaResult<TaskReference>
    where
        'sched: 'fut, // Scheduler must live at least as long as the future
        R: ToLuaMulti<'static>,
        F: 'static + Fn(&'static Lua) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>,
    {
        self.queue_async_task(thread_or_function, None, None, async move {
            match func(self.lua).await {
                Ok(res) => match res.to_lua_multi(self.lua) {
                    Ok(multi) => Ok(Some(multi)),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        })
    }

    /**
        Creates a function callable from Lua that runs an async
        closure and returns the results of it to the call site.
    */
    pub fn make_scheduled_async_fn<A, R, F, FR>(&self, func: F) -> LuaResult<LuaFunction>
    where
        A: FromLuaMulti<'static>,
        R: ToLuaMulti<'static>,
        F: 'static + Fn(&'static Lua, A) -> FR,
        FR: 'static + Future<Output = LuaResult<R>>,
    {
        let async_env_thread: LuaFunction = self.lua.named_registry_value("co.thread")?;
        let async_env_yield: LuaFunction = self.lua.named_registry_value("co.yield")?;
        self.lua
            .load(TASK_ASYNC_IMPL_LUA)
            .set_environment(
                TableBuilder::new(self.lua)?
                    .with_value("thread", async_env_thread)?
                    .with_value("yield", async_env_yield)?
                    .with_function(
                        "resumeAsync",
                        move |lua: &Lua, (thread, args): (LuaThread, A)| {
                            let fut = func(lua, args);
                            let sched = lua.app_data_ref::<&TaskScheduler>().unwrap();
                            sched.queue_async_task(LuaValue::Thread(thread), None, None, async {
                                let rets = fut.await?;
                                let mult = rets.to_lua_multi(lua)?;
                                Ok(Some(mult))
                            })
                        },
                    )?
                    .build_readonly()?,
            )?
            .into_function()
    }

    /**
        Registers a new background task with the task scheduler.

        This will ensure that the task scheduler keeps running until a
        call to [`TaskScheduler::deregister_background_task`] is made.

        The returned [`TaskSchedulerUnregistrar::unregister`]
        must be called upon completion of the background task to
        prevent the task scheduler from running indefinitely.
    */
    pub fn register_background_task(&self) -> TaskSchedulerAsyncHandle {
        let sender = self.futures_tx.clone();
        sender
            .send(TaskSchedulerMessage::Spawned)
            .unwrap_or_else(|e| {
                panic!(
                    "\
                    \nFailed to unregister background task - this is an internal error! \
                    \nPlease report it at {} \
                    \nDetails: {e} \
                    ",
                    env!("CARGO_PKG_REPOSITORY")
                )
            });
        TaskSchedulerAsyncHandle::new(sender)
    }

    /**
        Resumes the next queued Lua task, if one exists, blocking
        the current thread until it either yields or finishes.
    */
    fn resume_next_blocking_task(
        &self,
        override_args: Option<LuaResult<LuaMultiValue>>,
    ) -> TaskSchedulerState {
        match {
            let mut queue_guard = self.tasks_queue_blocking.borrow_mut();
            let task = queue_guard.pop_front();
            drop(queue_guard);
            task
        } {
            None => TaskSchedulerState::new(self),
            Some(task) => match self.resume_task(task, override_args) {
                Ok(_) => TaskSchedulerState::new(self),
                Err(task_err) => TaskSchedulerState::err(self, task_err),
            },
        }
    }

    /**
        Awaits the first available queued future, and resumes its associated
        Lua task which will be ready for resumption when that future wakes.

        Panics if there are no futures currently queued.

        Use [`TaskScheduler::next_queue_future_exists`]
        to check if there are any queued futures.
    */
    async fn resume_next_async_task(&self) -> TaskSchedulerState {
        let (task, result) = {
            let mut futs = self
                .futures
                .try_lock()
                .expect("Tried to resume next queued future while already resuming or modifying");
            futs.next()
                .await
                .expect("Tried to resume next queued future but none are queued")
        };
        // Promote this future task to a blocking task and resume it
        // right away, also taking care to not borrow mutably twice
        // by dropping this guard before trying to resume it
        let mut queue_guard = self.tasks_queue_blocking.borrow_mut();
        queue_guard.push_front(task);
        drop(queue_guard);
        self.resume_next_blocking_task(result.transpose())
    }

    /**
        Awaits the next background task registration
        message, if any messages exist in the queue.

        This is a no-op if there are no background tasks left running
        and / or the background task messages channel was closed.
    */
    async fn receive_next_message(&self) -> TaskSchedulerState {
        let message_opt = {
            let mut rx = self.futures_rx.lock().await;
            rx.recv().await
        };
        if let Some(message) = message_opt {
            match message {
                TaskSchedulerMessage::NewBlockingTaskReady => TaskSchedulerState::new(self),
                TaskSchedulerMessage::Spawned => {
                    let prev = self.futures_registered_count.get();
                    self.futures_registered_count.set(prev + 1);
                    TaskSchedulerState::new(self)
                }
                TaskSchedulerMessage::Terminated(result) => {
                    let prev = self.futures_registered_count.get();
                    self.futures_registered_count.set(prev - 1);
                    if prev == 0 {
                        panic!(
                            r#"
                            Terminated a background task without it running - this is an internal error!
                            Please report it at {}
                            "#,
                            env!("CARGO_PKG_REPOSITORY")
                        )
                    }
                    if let Err(e) = result {
                        TaskSchedulerState::err(self, e)
                    } else {
                        TaskSchedulerState::new(self)
                    }
                }
            }
        } else {
            TaskSchedulerState::new(self)
        }
    }

    /**
        Resumes the task scheduler queue.

        This will run any spawned or deferred Lua tasks in a blocking manner.

        Once all spawned and / or deferred Lua tasks have finished running,
        this will process delayed tasks, waiting tasks, and native Rust
        futures concurrently, awaiting the first one to be ready for resumption.
    */
    pub async fn resume_queue(&self) -> TaskSchedulerState {
        let current = TaskSchedulerState::new(self);
        /*
            Resume tasks in the internal queue, in this order:

            * 🛑 = blocking - lua tasks, in order
            * ⏳ = async - first come, first serve

            1. 🛑 Tasks from task.spawn / task.defer, the main thread
            2. ⏳ Tasks from task.delay / task.wait, spawned background tasks
        */
        if current.has_blocking_tasks() {
            self.resume_next_blocking_task(None)
        } else if current.has_future_tasks() && current.has_background_tasks() {
            // Futures, spawned background tasks
            tokio::select! {
                result = self.resume_next_async_task() => result,
                result = self.receive_next_message() => result,
            }
        } else if current.has_future_tasks() {
            // Futures
            self.resume_next_async_task().await
        } else if current.has_background_tasks() {
            // Only spawned background tasks, these may then
            // spawn new lua tasks and "wake up" the scheduler
            self.receive_next_message().await
        } else {
            TaskSchedulerState::new(self)
        }
    }
}
