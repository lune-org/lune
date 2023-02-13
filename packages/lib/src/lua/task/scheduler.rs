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

use mlua::prelude::*;

use tokio::time::{sleep, Instant};

type TaskSchedulerQueue = Arc<Mutex<VecDeque<TaskReference>>>;

/// An enum representing different kinds of tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskKind {
    Instant,
    Deferred,
    Yielded,
}

impl fmt::Display for TaskKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name: &'static str = match self {
            TaskKind::Instant => "Instant",
            TaskKind::Deferred => "Deferred",
            TaskKind::Yielded => "Yielded",
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
    queued_target: Option<Instant>,
}

impl TaskReference {
    pub const fn new(kind: TaskKind, guid: usize, queued_target: Option<Instant>) -> Self {
        Self {
            kind,
            guid,
            queued_target,
        }
    }
}

impl fmt::Display for TaskReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TaskReference({} - {})", self.kind, self.guid)
    }
}

impl LuaUserData for TaskReference {}

impl From<&Task> for TaskReference {
    fn from(value: &Task) -> Self {
        Self::new(value.kind, value.guid, value.queued_target)
    }
}

/// A struct representing a task contained in the task scheduler
#[derive(Debug)]
pub struct Task {
    kind: TaskKind,
    guid: usize,
    thread: LuaRegistryKey,
    args: LuaRegistryKey,
    queued_at: Instant,
    queued_target: Option<Instant>,
}

/// A struct representing the current status of the task scheduler
#[derive(Debug, Clone, Copy)]
pub struct TaskSchedulerStatus {
    pub exit_code: Option<ExitCode>,
    pub num_instant: usize,
    pub num_deferred: usize,
    pub num_yielded: usize,
    pub num_total: usize,
}

impl fmt::Display for TaskSchedulerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TaskSchedulerStatus(\nInstant: {}\nDeferred: {}\nYielded: {}\nTotal: {})",
            self.num_instant, self.num_deferred, self.num_yielded, self.num_total
        )
    }
}

/// A task scheduler that implements task queues
/// with instant, deferred, and delayed tasks
#[derive(Debug)]
pub struct TaskScheduler {
    lua: &'static Lua,
    guid: AtomicUsize,
    running: bool,
    tasks: Arc<Mutex<HashMap<TaskReference, Task>>>,
    task_queue_instant: TaskSchedulerQueue,
    task_queue_deferred: TaskSchedulerQueue,
    task_queue_yielded: TaskSchedulerQueue,
    exit_code_set: AtomicBool,
    exit_code: Arc<Mutex<ExitCode>>,
}

impl TaskScheduler {
    pub fn new(lua: &'static Lua) -> LuaResult<Self> {
        Ok(Self {
            lua,
            guid: AtomicUsize::new(0),
            running: false,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            task_queue_instant: Arc::new(Mutex::new(VecDeque::new())),
            task_queue_deferred: Arc::new(Mutex::new(VecDeque::new())),
            task_queue_yielded: Arc::new(Mutex::new(VecDeque::new())),
            exit_code_set: AtomicBool::new(false),
            exit_code: Arc::new(Mutex::new(ExitCode::SUCCESS)),
        })
    }

    pub fn into_static(self) -> &'static Self {
        Box::leak(Box::new(self))
    }

    pub fn status(&self) -> TaskSchedulerStatus {
        let counts = {
            (
                self.task_queue_instant.lock().unwrap().len(),
                self.task_queue_deferred.lock().unwrap().len(),
                self.task_queue_yielded.lock().unwrap().len(),
            )
        };
        let num_total = counts.0 + counts.1 + counts.2;
        let exit_code = if self.exit_code_set.load(Ordering::Relaxed) {
            Some(*self.exit_code.lock().unwrap())
        } else {
            None
        };
        TaskSchedulerStatus {
            exit_code,
            num_instant: counts.0,
            num_deferred: counts.1,
            num_yielded: counts.2,
            num_total,
        }
    }

    pub fn set_exit_code(&self, code: ExitCode) {
        self.exit_code_set.store(true, Ordering::Relaxed);
        *self.exit_code.lock().unwrap() = code
    }

    fn schedule<'a>(
        &self,
        kind: TaskKind,
        tof: LuaValue<'a>,
        args: Option<LuaMultiValue<'a>>,
        delay: Option<f64>,
    ) -> LuaResult<TaskReference> {
        // Get or create a thread from the given argument
        let task_thread = match tof {
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
        let task_args_vec = args.map(|opt| opt.into_vec());
        let task_thread_key = self.lua.create_registry_value(task_thread)?;
        let task_args_key = self.lua.create_registry_value(task_args_vec)?;
        // Create the full task struct
        let guid = self.guid.fetch_add(1, Ordering::Relaxed) + 1;
        let queued_at = Instant::now();
        let queued_target = delay.map(|secs| queued_at + Duration::from_secs_f64(secs));
        let task = Task {
            kind,
            guid,
            thread: task_thread_key,
            args: task_args_key,
            queued_at,
            queued_target,
        };
        // Create the task ref (before adding the task to the scheduler)
        let task_ref = TaskReference::from(&task);
        // Add it to the scheduler
        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(task_ref, task);
        }
        match kind {
            TaskKind::Instant => {
                // If we have a currently running task and we spawned an
                // instant task here it should run right after the currently
                // running task, so put it at the front of the task queue
                let mut queue = self.task_queue_instant.lock().unwrap();
                if self.running {
                    queue.push_front(task_ref);
                } else {
                    queue.push_back(task_ref);
                }
            }
            TaskKind::Deferred => {
                // Deferred tasks should always schedule
                // at the very end of the deferred queue
                let mut queue = self.task_queue_deferred.lock().unwrap();
                queue.push_back(task_ref);
            }
            TaskKind::Yielded => {
                // Find the first task that is scheduled after this one and insert before it,
                // this will ensure that our list of delayed tasks is sorted and we can grab
                // the very first one to figure out how long to yield until the next cycle
                let mut queue = self.task_queue_yielded.lock().unwrap();
                let idx = queue
                    .iter()
                    .enumerate()
                    .find_map(|(idx, t)| {
                        if t.queued_target > queued_target {
                            Some(idx)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(queue.len());
                queue.insert(idx, task_ref);
            }
        }
        Ok(task_ref)
    }

    pub fn schedule_instant<'a>(
        &self,
        tof: LuaValue<'a>,
        args: LuaMultiValue<'a>,
    ) -> LuaResult<TaskReference> {
        self.schedule(TaskKind::Instant, tof, Some(args), None)
    }

    pub fn schedule_deferred<'a>(
        &self,
        tof: LuaValue<'a>,
        args: LuaMultiValue<'a>,
    ) -> LuaResult<TaskReference> {
        self.schedule(TaskKind::Deferred, tof, Some(args), None)
    }

    pub fn schedule_delayed<'a>(
        &self,
        secs: f64,
        tof: LuaValue<'a>,
        args: LuaMultiValue<'a>,
    ) -> LuaResult<TaskReference> {
        self.schedule(TaskKind::Yielded, tof, Some(args), Some(secs))
    }

    pub fn resume_after(&self, secs: f64, thread: LuaThread<'_>) -> LuaResult<TaskReference> {
        self.schedule(
            TaskKind::Yielded,
            LuaValue::Thread(thread),
            None,
            Some(secs),
        )
    }

    pub fn cancel(&self, reference: TaskReference) -> bool {
        let queue_mutex = match reference.kind {
            TaskKind::Instant => &self.task_queue_instant,
            TaskKind::Deferred => &self.task_queue_deferred,
            TaskKind::Yielded => &self.task_queue_yielded,
        };
        let mut queue = queue_mutex.lock().unwrap();
        let mut found = false;
        queue.retain(|task| {
            if task.guid == reference.guid {
                found = true;
                false
            } else {
                true
            }
        });
        found
    }

    pub fn resume_task(&self, reference: TaskReference) -> LuaResult<()> {
        let task = {
            let mut tasks = self.tasks.lock().unwrap();
            match tasks.remove(&reference) {
                Some(task) => task,
                None => {
                    return Err(LuaError::RuntimeError(format!(
                        "Task does not exist in scheduler: {reference}"
                    )))
                }
            }
        };
        let thread: LuaThread = self.lua.registry_value(&task.thread)?;
        let args: Option<Vec<LuaValue>> = self.lua.registry_value(&task.args)?;
        if let Some(args) = args {
            thread.resume::<_, LuaMultiValue>(LuaMultiValue::from_vec(args))?;
        } else {
            let elapsed = task.queued_at.elapsed().as_secs_f64();
            thread.resume::<_, LuaMultiValue>(elapsed)?;
        }
        self.lua.remove_registry_value(task.thread)?;
        self.lua.remove_registry_value(task.args)?;
        Ok(())
    }

    fn get_queue(&self, kind: TaskKind) -> &TaskSchedulerQueue {
        match kind {
            TaskKind::Instant => &self.task_queue_instant,
            TaskKind::Deferred => &self.task_queue_deferred,
            TaskKind::Yielded => &self.task_queue_yielded,
        }
    }

    fn next_queue_task(&self, kind: TaskKind) -> Option<TaskReference> {
        let task = {
            let queue_guard = self.get_queue(kind).lock().unwrap();
            queue_guard.front().copied()
        };
        task
    }

    fn resume_next_queue_task(&self, kind: TaskKind) -> Option<LuaResult<TaskSchedulerStatus>> {
        match {
            let mut queue_guard = self.get_queue(kind).lock().unwrap();
            queue_guard.pop_front()
        } {
            None => {
                let status = self.status();
                if status.num_total > 0 {
                    Some(Ok(status))
                } else {
                    None
                }
            }
            Some(t) => match self.resume_task(t) {
                Ok(_) => Some(Ok(self.status())),
                Err(e) => Some(Err(e)),
            },
        }
    }

    pub async fn resume_queue(&self) -> Option<LuaResult<TaskSchedulerStatus>> {
        let now = Instant::now();
        let status = self.status();
        /*
            Resume tasks in the internal queue, in this order:

            1. Tasks from task.spawn, this includes the main thread
            2. Tasks from task.defer
            3. Tasks from task.delay OR futures, whichever comes first
            4. Tasks from futures
        */
        if status.num_instant > 0 {
            self.resume_next_queue_task(TaskKind::Instant)
        } else if status.num_deferred > 0 {
            self.resume_next_queue_task(TaskKind::Deferred)
        } else if status.num_yielded > 0 {
            // 3. Threads from task.delay or task.wait, futures
            let next_yield_target = self
                .next_queue_task(TaskKind::Yielded)
                .expect("Yielded task missing but status count is > 0")
                .queued_target
                .expect("Yielded task is missing queued target");
            // Resume this yielding task if its target time has passed
            if now >= next_yield_target {
                self.resume_next_queue_task(TaskKind::Yielded)
            } else {
                /*
                    Await the first future to be ready

                    - If it is the sleep fut then we will return and the next
                      call to resume_queue will then resume that yielded task

                    - If it is a future then we resume the corresponding task
                      that is has stored in the future-specific task queue
                */
                sleep(next_yield_target - now).await;
                // TODO: Implement this, for now we only await sleep
                // since the task scheduler doesn't support futures
                Some(Ok(self.status()))
            }
        } else {
            // 4. Just futures

            // TODO: Await the first future to be ready
            // and resume the corresponding task for it
            None
        }
    }
}
