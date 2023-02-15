use core::panic;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    process::ExitCode,
};

use futures_util::{future::LocalBoxFuture, stream::FuturesUnordered, Future};
use mlua::prelude::*;

use tokio::{
    sync::{mpsc, Mutex as AsyncMutex},
    time::Instant,
};

use super::message::TaskSchedulerMessage;
pub use super::{task_kind::TaskKind, task_reference::TaskReference};

type TaskFutureRets<'fut> = LuaResult<Option<LuaMultiValue<'fut>>>;
type TaskFuture<'fut> = LocalBoxFuture<'fut, (TaskReference, TaskFutureRets<'fut>)>;

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
    pub(super) lua: &'static Lua,
    pub(super) guid: Cell<usize>,
    pub(super) guid_running: Cell<Option<usize>>,
    pub(super) exit_code: Cell<Option<ExitCode>>,
    // Blocking tasks
    pub(super) tasks: RefCell<HashMap<TaskReference, Task>>,
    pub(super) tasks_queue_blocking: RefCell<VecDeque<TaskReference>>,
    // Future tasks & objects for waking
    pub(super) futures: AsyncMutex<FuturesUnordered<TaskFuture<'fut>>>,
    pub(super) futures_registered_count: Cell<usize>,
    pub(super) futures_tx: mpsc::UnboundedSender<TaskSchedulerMessage>,
    pub(super) futures_rx: AsyncMutex<mpsc::UnboundedReceiver<TaskSchedulerMessage>>,
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
}
