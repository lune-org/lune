use core::panic;
use std::{
    collections::{HashMap, VecDeque},
    fmt,
    process::ExitCode,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use futures_util::{future::LocalBoxFuture, stream::FuturesUnordered, Future, StreamExt};
use mlua::prelude::*;

use tokio::{
    sync::Mutex as AsyncMutex,
    time::{sleep, Instant},
};

type TaskSchedulerQueue = Arc<Mutex<VecDeque<TaskReference>>>;

type TaskFutureArgsOverride<'fut> = Option<Vec<LuaValue<'fut>>>;
type TaskFutureReturns<'fut> = LuaResult<TaskFutureArgsOverride<'fut>>;
type TaskFuture<'fut> = LocalBoxFuture<'fut, (TaskReference, TaskFutureReturns<'fut>)>;

/// An enum representing different kinds of tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskKind {
    Instant,
    Deferred,
    Future,
}

impl fmt::Display for TaskKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: &'static str = match self {
            TaskKind::Instant => "Instant",
            TaskKind::Deferred => "Deferred",
            TaskKind::Future => "Future",
        };
        write!(f, "{name}")
    }
}

/// A lightweight, clonable struct that represents a
/// task in the scheduler and is accessible from Lua
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskReference {
    kind: TaskKind,
    guid: usize,
}

impl TaskReference {
    pub const fn new(kind: TaskKind, guid: usize) -> Self {
        Self { kind, guid }
    }
}

impl fmt::Display for TaskReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TaskReference({} - {})", self.kind, self.guid)
    }
}

impl LuaUserData for TaskReference {}

/// A struct representing a task contained in the task scheduler
#[derive(Debug)]
pub struct Task {
    thread: LuaRegistryKey,
    args: LuaRegistryKey,
    queued_at: Instant,
}

/// A struct representing the current status of the task scheduler
#[derive(Debug, Clone, Copy)]
pub struct TaskSchedulerState {
    pub exit_code: Option<ExitCode>,
    pub num_instant: usize,
    pub num_deferred: usize,
    pub num_future: usize,
    pub num_total: usize,
}

impl fmt::Display for TaskSchedulerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TaskSchedulerStatus(\nInstant: {}\nDeferred: {}\nYielded: {}\nTotal: {})",
            self.num_instant, self.num_deferred, self.num_future, self.num_total
        )
    }
}

#[derive(Debug, Clone)]
pub enum TaskSchedulerResult {
    Finished {
        state: TaskSchedulerState,
    },
    TaskErrored {
        error: LuaError,
        state: TaskSchedulerState,
    },
    TaskSuccessful {
        state: TaskSchedulerState,
    },
}

/// A task scheduler that implements task queues
/// with instant, deferred, and delayed tasks
#[derive(Debug)]
pub struct TaskScheduler<'fut> {
    lua: &'static Lua,
    tasks: Arc<Mutex<HashMap<TaskReference, Task>>>,
    futures: Arc<AsyncMutex<FuturesUnordered<TaskFuture<'fut>>>>,
    task_queue_instant: TaskSchedulerQueue,
    task_queue_deferred: TaskSchedulerQueue,
    exit_code_set: AtomicBool,
    exit_code: Arc<Mutex<ExitCode>>,
    guid: AtomicUsize,
    guid_running_task: AtomicUsize,
}

impl<'fut> TaskScheduler<'fut> {
    /**
        Creates a new task scheduler.
    */
    pub fn new(lua: &'static Lua) -> LuaResult<Self> {
        Ok(Self {
            lua,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            futures: Arc::new(AsyncMutex::new(FuturesUnordered::new())),
            task_queue_instant: Arc::new(Mutex::new(VecDeque::new())),
            task_queue_deferred: Arc::new(Mutex::new(VecDeque::new())),
            exit_code_set: AtomicBool::new(false),
            exit_code: Arc::new(Mutex::new(ExitCode::SUCCESS)),
            // Global ids must start at 1, since 0 is a special
            // value for guid_running_task that means "no task"
            guid: AtomicUsize::new(1),
            guid_running_task: AtomicUsize::new(0),
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
        Gets the current state of the task scheduler.

        Panics if called during any of the task scheduler resumption phases.
    */
    pub fn state(&self) -> TaskSchedulerState {
        const MESSAGE: &str =
            "Failed to get lock - make sure not to call during task scheduler resumption";
        TaskSchedulerState {
            exit_code: if self.exit_code_set.load(Ordering::Relaxed) {
                Some(*self.exit_code.try_lock().expect(MESSAGE))
            } else {
                None
            },
            num_instant: self.task_queue_instant.try_lock().expect(MESSAGE).len(),
            num_deferred: self.task_queue_deferred.try_lock().expect(MESSAGE).len(),
            num_future: self.futures.try_lock().expect(MESSAGE).len(),
            num_total: self.tasks.try_lock().expect(MESSAGE).len(),
        }
    }

    /**
        Stores the exit code for the task scheduler.

        This will be passed back to the Rust thread that is running the task scheduler,
        in the [`TaskSchedulerState`] returned on resumption of the task scheduler queue.

        Setting this exit code will signal to that thread that it
        should stop resuming tasks, and gracefully terminate the program.
    */
    pub fn set_exit_code(&self, code: ExitCode) {
        *self.exit_code.lock().unwrap() = code;
        self.exit_code_set.store(true, Ordering::Relaxed);
    }

    /**
        Creates a new task, storing a new Lua thread
        for it, as well as the arguments to give the
        thread on resumption, in the Lua registry.
    */
    fn create_task(
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
            let guid = self.guid.fetch_add(1, Ordering::Relaxed);
            TaskReference::new(kind, guid)
        };
        // Add the task to the scheduler
        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(task_ref, task);
        }
        Ok(task_ref)
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
    fn queue_task(
        &self,
        kind: TaskKind,
        thread_or_function: LuaValue<'_>,
        thread_args: Option<LuaMultiValue<'_>>,
        guid_to_reuse: Option<usize>,
        after_current_resume: bool,
    ) -> LuaResult<TaskReference> {
        if kind == TaskKind::Future {
            panic!("Tried to schedule future using normal task schedule method")
        }
        let task_ref = self.create_task(kind, thread_or_function, thread_args, guid_to_reuse)?;
        match kind {
            TaskKind::Instant => {
                let mut queue = self.task_queue_instant.lock().unwrap();
                if after_current_resume {
                    assert!(
                        queue.len() > 0,
                        "Cannot schedule a task after the first instant when task queue is empty"
                    );
                    queue.insert(1, task_ref);
                } else {
                    queue.push_front(task_ref);
                }
            }
            TaskKind::Deferred => {
                // Deferred tasks should always schedule at the end of the deferred queue
                let mut queue = self.task_queue_deferred.lock().unwrap();
                queue.push_back(task_ref);
            }
            TaskKind::Future => unreachable!(),
        }
        Ok(task_ref)
    }

    /**
        Queues a new future to run on the task scheduler.
    */
    fn queue_async(
        &self,
        thread_or_function: LuaValue<'_>,
        thread_args: Option<LuaMultiValue<'_>>,
        guid_to_reuse: Option<usize>,
        fut: impl Future<Output = TaskFutureReturns<'fut>> + 'fut,
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
    pub fn schedule_current_resume(
        &self,
        thread_or_function: LuaValue<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference> {
        self.queue_task(
            TaskKind::Instant,
            thread_or_function,
            Some(thread_args),
            None,
            false,
        )
    }

    /**
        Schedules a lua thread or function to resume ***after the first***
        currently resuming task, during this resumption point.

        The given lua thread or function will be resumed
        using the given `thread_args` as its argument(s).
    */
    pub fn schedule_after_current_resume(
        &self,
        thread_or_function: LuaValue<'_>,
        thread_args: LuaMultiValue<'_>,
    ) -> LuaResult<TaskReference> {
        self.queue_task(
            TaskKind::Instant,
            thread_or_function,
            Some(thread_args),
            // This should recycle the guid of the current task,
            // since it will only be called to schedule resuming
            // current thread after it gives resumption to another
            match self.guid_running_task.load(Ordering::Relaxed) {
                0 => panic!("Tried to schedule with no task running"),
                guid => Some(guid),
            },
            true,
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
        self.queue_task(
            TaskKind::Deferred,
            thread_or_function,
            Some(thread_args),
            None,
            false,
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
        self.queue_async(thread_or_function, Some(thread_args), None, async move {
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
        self.queue_async(
            thread_or_function,
            None,
            // Wait should recycle the guid of the current task,
            // which ensures that the TaskReference is identical and
            // that any waits inside of spawned tasks will also cancel
            match self.guid_running_task.load(Ordering::Relaxed) {
                0 => panic!("Tried to schedule waiting task with no task running"),
                guid => Some(guid),
            },
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
    pub fn schedule_async(
        &self,
        thread_or_function: LuaValue<'_>,
        fut: impl Future<Output = TaskFutureReturns<'fut>> + 'fut,
    ) -> LuaResult<TaskReference> {
        self.queue_async(thread_or_function, None, None, fut)
    }

    /**
        Checks if a task still exists in the scheduler.

        A task may no longer exist in the scheduler if it has been manually
        cancelled and removed by calling [`TaskScheduler::cancel_task()`].
    */
    #[allow(dead_code)]
    pub fn contains_task(&self, reference: TaskReference) -> bool {
        let tasks = self.tasks.lock().unwrap();
        tasks.contains_key(&reference)
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
        let mut tasks = self.tasks.lock().unwrap();
        // Unfortunately we have to loop through to find which task
        // references to remove instead of removing directly since
        // tasks can switch kinds between instant, deferred, future
        let tasks_to_remove: Vec<_> = tasks
            .keys()
            .filter(|task_ref| task_ref.guid == reference.guid)
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
    pub fn resume_task(
        &self,
        reference: TaskReference,
        override_args: Option<Vec<LuaValue>>,
    ) -> LuaResult<()> {
        self.guid_running_task
            .store(reference.guid, Ordering::Relaxed);
        let task = {
            let mut tasks = self.tasks.lock().unwrap();
            match tasks.remove(&reference) {
                Some(task) => task,
                None => return Ok(()), // Task was removed
            }
        };
        let thread: LuaThread = self.lua.registry_value(&task.thread)?;
        let args_vec_opt = override_args.or_else(|| {
            self.lua
                .registry_value::<Option<Vec<LuaValue>>>(&task.args)
                .expect("Failed to get stored args for task")
        });
        self.lua.remove_registry_value(task.thread)?;
        self.lua.remove_registry_value(task.args)?;
        if let Some(args) = args_vec_opt {
            thread.resume::<_, LuaMultiValue>(LuaMultiValue::from_vec(args))?;
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
            thread.resume::<_, LuaMultiValue>(elapsed)?;
        }
        self.guid_running_task.store(0, Ordering::Relaxed);
        Ok(())
    }

    /**
        Retrieves the queue for a specific kind of task.

        Panics for [`TaskKind::Future`] since
        futures do not use the normal task queue.
    */
    fn get_queue(&self, kind: TaskKind) -> &TaskSchedulerQueue {
        match kind {
            TaskKind::Instant => &self.task_queue_instant,
            TaskKind::Deferred => &self.task_queue_deferred,
            TaskKind::Future => {
                panic!("Future tasks do not use the normal task queue")
            }
        }
    }

    /**
        Checks if a future exists in the task queue.

        Panics if called during resumption of the futures task queue.
    */
    fn next_queue_future_exists(&self) -> bool {
        let futs = self.futures.try_lock().expect(
            "Failed to get lock on futures - make sure not to call during futures resumption",
        );
        !futs.is_empty()
    }

    /**
        Resumes the next queued Lua task, if one exists, blocking
        the current thread until it either yields or finishes.
    */
    fn resume_next_queue_task(
        &self,
        kind: TaskKind,
        override_args: Option<Vec<LuaValue>>,
    ) -> TaskSchedulerResult {
        match {
            let mut queue_guard = self.get_queue(kind).lock().unwrap();
            queue_guard.pop_front()
        } {
            None => {
                let status = self.state();
                if status.num_total > 0 {
                    TaskSchedulerResult::TaskSuccessful {
                        state: self.state(),
                    }
                } else {
                    TaskSchedulerResult::Finished {
                        state: self.state(),
                    }
                }
            }
            Some(task) => match self.resume_task(task, override_args) {
                Ok(()) => TaskSchedulerResult::TaskSuccessful {
                    state: self.state(),
                },
                Err(task_err) => TaskSchedulerResult::TaskErrored {
                    error: task_err,
                    state: self.state(),
                },
            },
        }
    }

    /**
        Awaits the first available queued future, and resumes its
        associated Lua task which will then be ready for resumption.

        Panics if there are no futures currently queued.

        Use [`TaskScheduler::next_queue_future_exists`]
        to check if there are any queued futures.
    */
    async fn resume_next_queue_future(&self) -> TaskSchedulerResult {
        let result = {
            let mut futs = self
                .futures
                .try_lock()
                .expect("Failed to get lock on futures");
            futs.next()
                .await
                .expect("Tried to resume next queued future but none are queued")
        };
        match result {
            (task, Err(fut_err)) => {
                // Future errored, don't resume its associated task
                // and make sure to cancel / remove it completely
                let error_prefer_cancel = match self.remove_task(task) {
                    Err(cancel_err) => cancel_err,
                    Ok(_) => fut_err,
                };
                TaskSchedulerResult::TaskErrored {
                    error: error_prefer_cancel,
                    state: self.state(),
                }
            }
            (task, Ok(args)) => {
                // Promote this future task to an instant task
                // and resume the instant queue right away, taking
                // care to not deadlock by dropping the mutex guard
                let mut queue_guard = self.get_queue(TaskKind::Instant).lock().unwrap();
                queue_guard.push_front(task);
                drop(queue_guard);
                self.resume_next_queue_task(TaskKind::Instant, args)
            }
        }
    }

    /**
        Resumes the task scheduler queue.

        This will run any spawned or deferred Lua tasks in a blocking manner.

        Once all spawned and / or deferred Lua tasks have finished running,
        this will process delayed tasks, waiting tasks, and native Rust
        futures concurrently, awaiting the first one to be ready for resumption.
    */
    pub async fn resume_queue(&self) -> TaskSchedulerResult {
        let status = self.state();
        /*
            Resume tasks in the internal queue, in this order:

            1. Tasks from task.spawn, this includes the main thread
            2. Tasks from task.defer
            3. Tasks from task.delay / task.wait / native futures, first ready first resumed
        */
        if status.num_instant > 0 {
            self.resume_next_queue_task(TaskKind::Instant, None)
        } else if status.num_deferred > 0 {
            self.resume_next_queue_task(TaskKind::Deferred, None)
        } else {
            // 3. Threads from task.delay or task.wait, futures
            if self.next_queue_future_exists() {
                self.resume_next_queue_future().await
            } else {
                TaskSchedulerResult::Finished {
                    state: self.state(),
                }
            }
        }
    }
}
