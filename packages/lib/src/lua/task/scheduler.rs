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
    sync::{mpsc, Mutex as AsyncMutex},
    time::{sleep, Instant},
};

use crate::utils::table::TableBuilder;

type TaskSchedulerQueue = Arc<Mutex<VecDeque<TaskReference>>>;

type TaskFutureArgsOverride<'fut> = Option<Vec<LuaValue<'fut>>>;
type TaskFutureReturns<'fut> = LuaResult<TaskFutureArgsOverride<'fut>>;
type TaskFuture<'fut> = LocalBoxFuture<'fut, (TaskReference, TaskFutureReturns<'fut>)>;

const TASK_ASYNC_IMPL_LUA: &str = r#"
resume_async(thread(), ...)
return yield()
"#;

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

/// A lightweight, copyable struct that represents a
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

/**
    A handle to a registered background task.

    [`TaskSchedulerUnregistrar::unregister`] must be
    called upon completion of the background task to
    prevent the task scheduler from running indefinitely.
*/
#[must_use = "Background tasks must be unregistered"]
#[derive(Debug)]
pub struct TaskSchedulerBackgroundTaskHandle {
    sender: mpsc::UnboundedSender<TaskSchedulerRegistrationMessage>,
}

impl TaskSchedulerBackgroundTaskHandle {
    pub fn unregister(self, result: LuaResult<()>) {
        self.sender
            .send(TaskSchedulerRegistrationMessage::Terminated(result))
            .unwrap_or_else(|_| {
                panic!(
                    "\
                    \nFailed to unregister background task - this is an internal error! \
                    \nPlease report it at {} \
                    \nDetails: Manual \
                    ",
                    env!("CARGO_PKG_REPOSITORY")
                )
            });
    }
}

/// A struct representing the current state of the task scheduler
#[derive(Debug, Clone)]
pub struct TaskSchedulerResult {
    lua_error: Option<LuaError>,
    exit_code: Option<ExitCode>,
    num_instant: usize,
    num_deferred: usize,
    num_futures: usize,
    num_background: usize,
    num_active: usize,
}

impl TaskSchedulerResult {
    fn new(sched: &TaskScheduler) -> Self {
        const MESSAGE: &str =
            "Failed to get lock - make sure not to call during task scheduler resumption";
        Self {
            lua_error: None,
            exit_code: if sched.exit_code_set.load(Ordering::Relaxed) {
                Some(*sched.exit_code.try_lock().expect(MESSAGE))
            } else {
                None
            },
            num_instant: sched.task_queue_instant.try_lock().expect(MESSAGE).len(),
            num_deferred: sched.task_queue_deferred.try_lock().expect(MESSAGE).len(),
            num_futures: sched.futures.try_lock().expect(MESSAGE).len(),
            num_background: sched.futures_in_background.load(Ordering::Relaxed),
            num_active: sched.tasks.try_lock().expect(MESSAGE).len(),
        }
    }

    fn err(sched: &TaskScheduler, err: LuaError) -> Self {
        let mut this = Self::new(sched);
        this.lua_error = Some(err);
        this
    }

    /**
        Returns a clone of the error from
        this task scheduler result, if any.
    */
    pub fn get_lua_error(&self) -> Option<LuaError> {
        self.lua_error.clone()
    }

    /**
        Returns a clone of the exit code from
        this task scheduler result, if any.
    */
    pub fn get_exit_code(&self) -> Option<ExitCode> {
        self.exit_code
    }

    /**
        Returns `true` if the task scheduler is still busy,
        meaning it still has lua threads left to run.
    */
    #[allow(dead_code)]
    pub fn is_busy(&self) -> bool {
        self.num_active > 0
    }

    /**
        Returns `true` if the task scheduler has finished all
        lua threads, but still has background tasks running.
    */
    #[allow(dead_code)]
    pub fn is_background(&self) -> bool {
        self.num_active == 0 && self.num_background > 0
    }

    /**
        Returns `true` if the task scheduler is done,
        meaning it has no lua threads left to run, and
        no spawned tasks are running in the background.
    */
    pub fn is_done(&self) -> bool {
        self.num_active == 0 && self.num_background == 0
    }
}

impl fmt::Display for TaskSchedulerResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.is_busy() {
            "Busy"
        } else if self.is_background() {
            "Background"
        } else {
            "Done"
        };
        let code = match self.get_exit_code() {
            Some(code) => format!("{code:?}"),
            None => "-".to_string(),
        };
        let err = match self.get_lua_error() {
            Some(e) => format!("{e:?}")
                .as_bytes()
                .chunks(42) // Kinda arbitrary but should fit in most terminals
                .enumerate()
                .map(|(idx, buf)| {
                    format!(
                        "{}{}{}{}{}",
                        if idx == 0 { "" } else { "\n│ " },
                        if idx == 0 {
                            "".to_string()
                        } else {
                            " ".repeat(16)
                        },
                        if idx == 0 { "" } else { " │ " },
                        String::from_utf8_lossy(buf),
                        if buf.len() == 42 { " │" } else { "" },
                    )
                })
                .collect::<String>(),
            None => "-".to_string(),
        };
        let parts = vec![
            format!("Status           │ {status}"),
            format!("Tasks active     │ {}", self.num_active),
            format!("Tasks background │ {}", self.num_background),
            format!("Status code      │ {code}"),
            format!("Lua error        │ {err}"),
        ];
        let lengths = parts
            .iter()
            .map(|part| {
                part.lines()
                    .next()
                    .unwrap()
                    .trim_end_matches(" │")
                    .chars()
                    .count()
            })
            .collect::<Vec<_>>();
        let longest = &parts
            .iter()
            .enumerate()
            .fold(0, |acc, (index, _)| acc.max(lengths[index]));
        let sep = "─".repeat(longest + 2);
        writeln!(f, "┌{}┐", &sep)?;
        for (index, part) in parts.iter().enumerate() {
            writeln!(
                f,
                "│ {}{} │",
                part.trim_end_matches(" │"),
                " ".repeat(
                    longest
                        - part
                            .lines()
                            .last()
                            .unwrap()
                            .trim_end_matches(" │")
                            .chars()
                            .count()
                )
            )?;
            if index < parts.len() - 1 {
                writeln!(f, "┝{}┥", &sep)?;
            }
        }
        writeln!(f, "└{}┘", &sep)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TaskSchedulerRegistrationMessage {
    Spawned,
    Terminated(LuaResult<()>),
}

/// A task scheduler that implements task queues
/// with instant, deferred, and delayed tasks
#[derive(Debug)]
pub struct TaskScheduler<'fut> {
    lua: &'static Lua,
    tasks: Arc<Mutex<HashMap<TaskReference, Task>>>,
    futures: Arc<AsyncMutex<FuturesUnordered<TaskFuture<'fut>>>>,
    futures_tx: mpsc::UnboundedSender<TaskSchedulerRegistrationMessage>,
    futures_rx: Arc<AsyncMutex<mpsc::UnboundedReceiver<TaskSchedulerRegistrationMessage>>>,
    futures_in_background: AtomicUsize,
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
        let (tx, rx) = mpsc::unbounded_channel();
        Ok(Self {
            lua,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            futures: Arc::new(AsyncMutex::new(FuturesUnordered::new())),
            futures_tx: tx,
            futures_rx: Arc::new(AsyncMutex::new(rx)),
            futures_in_background: AtomicUsize::new(0),
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
    pub fn schedule_async(
        &self,
        thread_or_function: LuaValue<'_>,
        fut: impl Future<Output = TaskFutureReturns<'fut>> + 'fut,
    ) -> LuaResult<TaskReference> {
        self.queue_async(thread_or_function, None, None, fut)
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
                        "resume_async",
                        move |lua: &Lua, (thread, args): (LuaThread, A)| {
                            let fut = func(lua, args);
                            let sched = lua.app_data_mut::<&TaskScheduler>().unwrap();
                            sched.schedule_async(LuaValue::Thread(thread), async {
                                let rets = fut.await?;
                                let mult = rets.to_lua_multi(lua)?;
                                Ok(Some(mult.into_vec()))
                            })
                        },
                    )?
                    .build_readonly()?,
            )?
            .into_function()
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
        Registers a new background task with the task scheduler.

        This will ensure that the task scheduler keeps running until a
        call to [`TaskScheduler::deregister_background_task`] is made.

        The returned [`TaskSchedulerUnregistrar::unregister`]
        must be called upon completion of the background task to
        prevent the task scheduler from running indefinitely.
    */
    pub fn register_background_task(&self) -> TaskSchedulerBackgroundTaskHandle {
        let sender = self.futures_tx.clone();
        sender
            .send(TaskSchedulerRegistrationMessage::Spawned)
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
        TaskSchedulerBackgroundTaskHandle { sender }
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
            None => TaskSchedulerResult::new(self),
            Some(task) => match self.resume_task(task, override_args) {
                Ok(()) => TaskSchedulerResult::new(self),
                Err(task_err) => TaskSchedulerResult::err(self, task_err),
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
                // and make sure to cancel / remove it completely, if removal
                // also errors then we send that error back instead of the future's error
                TaskSchedulerResult::err(
                    self,
                    match self.remove_task(task) {
                        Err(cancel_err) => cancel_err,
                        Ok(_) => fut_err,
                    },
                )
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
        Awaits the next background task registration
        message, if any messages exist in the queue.

        This is a no-op if there are no messages.
    */
    async fn receive_next_message(&self) -> TaskSchedulerResult {
        let message_opt = {
            let mut rx = self.futures_rx.lock().await;
            rx.recv().await
        };
        if let Some(message) = message_opt {
            match message {
                TaskSchedulerRegistrationMessage::Spawned => {
                    self.futures_in_background.fetch_add(1, Ordering::Relaxed);
                    TaskSchedulerResult::new(self)
                }
                TaskSchedulerRegistrationMessage::Terminated(result) => {
                    let prev = self.futures_in_background.fetch_sub(1, Ordering::Relaxed);
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
                        TaskSchedulerResult::err(self, e)
                    } else {
                        TaskSchedulerResult::new(self)
                    }
                }
            }
        } else {
            TaskSchedulerResult::new(self)
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
        let current = TaskSchedulerResult::new(self);
        /*
            Resume tasks in the internal queue, in this order:

            * 🛑 = blocking - lua tasks, in order
            * ⏳ = async - first come, first serve

            1. 🛑 Tasks from task.spawn and the main thread
            2. 🛑 Tasks from task.defer
            3. ⏳ Tasks from task.delay / task.wait, spawned background tasks
        */
        if current.num_instant > 0 {
            self.resume_next_queue_task(TaskKind::Instant, None)
        } else if current.num_deferred > 0 {
            self.resume_next_queue_task(TaskKind::Deferred, None)
        } else if current.num_futures > 0 && current.num_background > 0 {
            // Futures, spawned background tasks
            tokio::select! {
                result = self.resume_next_queue_future() => result,
                result = self.receive_next_message() => result,
            }
        } else if current.num_futures > 0 {
            // Futures
            self.resume_next_queue_future().await
        } else if current.num_background > 0 {
            // Only spawned background tasks, these may then
            // spawn new lua tasks and "wake up" the scheduler
            self.receive_next_message().await
        } else {
            TaskSchedulerResult::new(self)
        }
    }
}
