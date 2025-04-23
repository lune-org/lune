#![allow(unused_imports)]
#![allow(clippy::missing_errors_doc)]

use std::{
    cell::Cell, future::Future, process::ExitCode, rc::Weak as WeakRc, sync::Weak as WeakArc,
};

use async_executor::{Executor, Task};
use mlua::prelude::*;
use tracing::trace;

use crate::{
    exit::Exit,
    queue::{DeferredThreadQueue, FuturesQueue, SpawnedThreadQueue},
    result_map::ThreadResultMap,
    scheduler::Scheduler,
    thread_id::ThreadId,
};

/**
    Trait for any struct that can be turned into an [`LuaThread`]
    and passed to the scheduler, implemented for the following types:

    - Lua threads ([`LuaThread`])
    - Lua functions ([`LuaFunction`])
    - Lua chunks ([`LuaChunk`])
*/
pub trait IntoLuaThread {
    /**
        Converts the value into a Lua thread.

        # Errors

        Errors when out of memory.
    */
    fn into_lua_thread(self, lua: &Lua) -> LuaResult<LuaThread>;
}

impl IntoLuaThread for LuaThread {
    fn into_lua_thread(self, _: &Lua) -> LuaResult<LuaThread> {
        Ok(self)
    }
}

impl IntoLuaThread for LuaFunction {
    fn into_lua_thread(self, lua: &Lua) -> LuaResult<LuaThread> {
        lua.create_thread(self)
    }
}

impl IntoLuaThread for LuaChunk<'_> {
    fn into_lua_thread(self, lua: &Lua) -> LuaResult<LuaThread> {
        lua.create_thread(self.into_function()?)
    }
}

impl<T> IntoLuaThread for &T
where
    T: IntoLuaThread + Clone,
{
    fn into_lua_thread(self, lua: &Lua) -> LuaResult<LuaThread> {
        self.clone().into_lua_thread(lua)
    }
}

/**
    Trait for interacting with the current [`Scheduler`].

    Provides extra methods on the [`Lua`] struct for:

    - Setting the exit code and forcibly stopping the scheduler
    - Pushing (spawning) and deferring (pushing to the back) lua threads
    - Tracking and getting the result of lua threads
*/
pub trait LuaSchedulerExt {
    /**
        Sets the exit code of the current scheduler.

        See [`Scheduler::set_exit_code`] for more information.

        # Panics

        Panics if called outside of a running [`Scheduler`].
    */
    fn set_exit_code(&self, code: u8);

    /**
        Pushes (spawns) a lua thread to the **front** of the current scheduler.

        See [`Scheduler::push_thread_front`] for more information.

        # Panics

        Panics if called outside of a running [`Scheduler`].
    */
    fn push_thread_front(
        &self,
        thread: impl IntoLuaThread,
        args: impl IntoLuaMulti,
    ) -> LuaResult<ThreadId>;

    /**
        Pushes (defers) a lua thread to the **back** of the current scheduler.

        See [`Scheduler::push_thread_back`] for more information.

        # Panics

        Panics if called outside of a running [`Scheduler`].
    */
    fn push_thread_back(
        &self,
        thread: impl IntoLuaThread,
        args: impl IntoLuaMulti,
    ) -> LuaResult<ThreadId>;

    /**
        Registers the given thread to be tracked within the current scheduler.

        Must be called before waiting for a thread to complete or getting its result.
    */
    fn track_thread(&self, id: ThreadId);

    /**
        Gets the result of the given thread.

        See [`Scheduler::get_thread_result`] for more information.

        # Panics

        Panics if called outside of a running [`Scheduler`].
    */
    fn get_thread_result(&self, id: ThreadId) -> Option<LuaResult<LuaMultiValue>>;

    /**
        Waits for the given thread to complete.

        See [`Scheduler::wait_for_thread`] for more information.

        # Panics

        Panics if called outside of a running [`Scheduler`].
    */
    fn wait_for_thread(&self, id: ThreadId) -> impl Future<Output = ()>;
}

/**
    Trait for interacting with the [`Executor`] for the current [`Scheduler`].

    Provides extra methods on the [`Lua`] struct for:

    - Spawning thread-local (`!Send`) futures on the current executor
    - Spawning background (`Send`) futures on the current executor
    - Spawning blocking tasks on a separate thread pool
*/
pub trait LuaSpawnExt {
    /**
        Spawns the given future on the current executor and returns its [`Task`].

        # Panics

        Panics if called outside of a running [`Scheduler`].

        # Example usage

        ```rust
        use async_io::block_on;

        use mlua::prelude::*;
        use mlua_luau_scheduler::*;

        fn main() -> LuaResult<()> {
            let lua = Lua::new();

            lua.globals().set(
                "spawnBackgroundTask",
                lua.create_async_function(|lua, ()| async move {
                    lua.spawn(async move {
                        println!("Hello from background task!");
                    }).await;
                    Ok(())
                })?
            )?;

            let sched = Scheduler::new(lua.clone());
            sched.push_thread_front(lua.load("spawnBackgroundTask()"), ());
            block_on(sched.run());

            Ok(())
        }
        ```
    */
    fn spawn<F, T>(&self, fut: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;

    /**
        Spawns the given thread-local future on the current executor.

        Note that this future will run detached and always to completion,
        preventing the [`Scheduler`] was spawned on from completing until done.

        # Panics

        Panics if called outside of a running [`Scheduler`].

        # Example usage

        ```rust
        use async_io::block_on;

        use mlua::prelude::*;
        use mlua_luau_scheduler::*;

        fn main() -> LuaResult<()> {
            let lua = Lua::new();

            lua.globals().set(
                "spawnLocalTask",
                lua.create_async_function(|lua, ()| async move {
                    lua.spawn_local(async move {
                        println!("Hello from local task!");
                    });
                    Ok(())
                })?
            )?;

            let sched = Scheduler::new(lua.clone());
            sched.push_thread_front(lua.load("spawnLocalTask()"), ());
            block_on(sched.run());

            Ok(())
        }
        ```
    */
    fn spawn_local<F>(&self, fut: F)
    where
        F: Future<Output = ()> + 'static;

    /**
        Spawns the given blocking function and returns its [`Task`].

        This function will run on a separate thread pool and not block the current executor.

        # Panics

        Panics if called outside of a running [`Scheduler`].

        # Example usage

        ```rust
        use async_io::block_on;

        use mlua::prelude::*;
        use mlua_luau_scheduler::*;

        fn main() -> LuaResult<()> {
            let lua = Lua::new();

            lua.globals().set(
                "spawnBlockingTask",
                lua.create_async_function(|lua, ()| async move {
                    lua.spawn_blocking(|| {
                        println!("Hello from blocking task!");
                    }).await;
                    Ok(())
                })?
            )?;

            let sched = Scheduler::new(lua.clone());
            sched.push_thread_front(lua.load("spawnBlockingTask()"), ());
            block_on(sched.run());

            Ok(())
        }
        ```
    */
    fn spawn_blocking<F, T>(&self, f: F) -> Task<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;
}

impl LuaSchedulerExt for Lua {
    fn set_exit_code(&self, code: u8) {
        let exit = self
            .app_data_ref::<Exit>()
            .expect("exit code can only be set from within an active scheduler");
        exit.set(code);
    }

    fn push_thread_front(
        &self,
        thread: impl IntoLuaThread,
        args: impl IntoLuaMulti,
    ) -> LuaResult<ThreadId> {
        let queue = self
            .app_data_ref::<SpawnedThreadQueue>()
            .expect("lua threads can only be pushed from within an active scheduler");
        queue.push_item(self, thread, args)
    }

    fn push_thread_back(
        &self,
        thread: impl IntoLuaThread,
        args: impl IntoLuaMulti,
    ) -> LuaResult<ThreadId> {
        let queue = self
            .app_data_ref::<DeferredThreadQueue>()
            .expect("lua threads can only be pushed from within an active scheduler");
        queue.push_item(self, thread, args)
    }

    fn track_thread(&self, id: ThreadId) {
        let map = self
            .app_data_ref::<ThreadResultMap>()
            .expect("lua threads can only be tracked from within an active scheduler");
        map.track(id);
    }

    fn get_thread_result(&self, id: ThreadId) -> Option<LuaResult<LuaMultiValue>> {
        let map = self
            .app_data_ref::<ThreadResultMap>()
            .expect("lua threads results can only be retrieved from within an active scheduler");
        map.remove(id).map(|r| r.value(self))
    }

    fn wait_for_thread(&self, id: ThreadId) -> impl Future<Output = ()> {
        let map = self
            .app_data_ref::<ThreadResultMap>()
            .expect("lua threads results can only be retrieved from within an active scheduler");
        async move { map.listen(id).await }
    }
}

impl LuaSpawnExt for Lua {
    fn spawn<F, T>(&self, fut: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let exec = self
            .app_data_ref::<WeakArc<Executor>>()
            .expect("tasks can only be spawned within an active scheduler")
            .upgrade()
            .expect("executor was dropped");
        trace!("spawning future on executor");
        exec.spawn(fut)
    }

    fn spawn_local<F>(&self, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        let queue = self
            .app_data_ref::<WeakRc<FuturesQueue>>()
            .expect("tasks can only be spawned within an active scheduler")
            .upgrade()
            .expect("executor was dropped");
        trace!("spawning local task on executor");
        queue.push_item(fut);
    }

    fn spawn_blocking<F, T>(&self, f: F) -> Task<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let exec = self
            .app_data_ref::<WeakArc<Executor>>()
            .expect("tasks can only be spawned within an active scheduler")
            .upgrade()
            .expect("executor was dropped");
        trace!("spawning blocking task on executor");
        exec.spawn(blocking::unblock(f))
    }
}
