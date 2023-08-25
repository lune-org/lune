use std::sync::Arc;

use mlua::prelude::*;

use super::{
    thread::{SchedulerThread, SchedulerThreadId, SchedulerThreadSender},
    IntoLuaThread, Scheduler,
};

impl<'fut> Scheduler<'fut> {
    /**
        Checks if there are any lua threads to run.
    */
    pub(super) fn has_thread(&self) -> bool {
        !self
            .threads
            .try_lock()
            .expect("Failed to lock threads vec")
            .is_empty()
    }

    /**
        Pops the next thread to run, from the front of the scheduler.

        Returns `None` if there are no threads left to run.
    */
    pub(super) fn pop_thread(&self) -> LuaResult<Option<SchedulerThread>> {
        match self
            .threads
            .try_lock()
            .into_lua_err()
            .context("Failed to lock threads vec")?
            .pop_front()
        {
            Some(thread) => Ok(Some(thread)),
            None => Ok(None),
        }
    }

    /**
        Schedules the `thread` to be resumed with the given [`LuaError`].
    */
    pub fn push_err<'a>(
        &self,
        lua: &'a Lua,
        thread: impl IntoLuaThread<'a>,
        err: LuaError,
    ) -> LuaResult<()> {
        let thread = thread.into_lua_thread(lua)?;
        let args = LuaMultiValue::new(); // Will be resumed with error, don't need real args

        let thread = SchedulerThread::new(lua, thread, args);
        let thread_id = thread.id();

        self.state.set_thread_error(thread_id, err);
        self.threads
            .try_lock()
            .into_lua_err()
            .context("Failed to lock threads vec")?
            .push_front(thread);

        // NOTE: We might be resuming futures, need to signal that a
        // new lua thread is ready to break out of futures resumption
        self.state.message_sender().send_pushed_lua_thread();

        Ok(())
    }

    /**
        Schedules the `thread` to be resumed with the given `args`
        right away, before any other currently scheduled threads.
    */
    pub fn push_front<'a>(
        &self,
        lua: &'a Lua,
        thread: impl IntoLuaThread<'a>,
        args: impl IntoLuaMulti<'a>,
    ) -> LuaResult<SchedulerThreadId> {
        let thread = thread.into_lua_thread(lua)?;
        let args = args.into_lua_multi(lua)?;

        let thread = SchedulerThread::new(lua, thread, args);
        let thread_id = thread.id();

        self.threads
            .try_lock()
            .into_lua_err()
            .context("Failed to lock threads vec")?
            .push_front(thread);

        // NOTE: We might be resuming the same thread several times and
        // pushing it to the scheduler several times before it is done,
        // and we should only ever create one result sender per thread
        self.thread_senders
            .try_lock()
            .into_lua_err()
            .context("Failed to lock thread senders vec")?
            .entry(thread_id)
            .or_insert_with(|| SchedulerThreadSender::new(1));

        // NOTE: We might be resuming futures, need to signal that a
        // new lua thread is ready to break out of futures resumption
        self.state.message_sender().send_pushed_lua_thread();

        Ok(thread_id)
    }

    /**
        Schedules the `thread` to be resumed with the given `args`
        after all other current threads have been resumed.
    */
    pub fn push_back<'a>(
        &self,
        lua: &'a Lua,
        thread: impl IntoLuaThread<'a>,
        args: impl IntoLuaMulti<'a>,
    ) -> LuaResult<SchedulerThreadId> {
        let thread = thread.into_lua_thread(lua)?;
        let args = args.into_lua_multi(lua)?;

        let thread = SchedulerThread::new(lua, thread, args);
        let thread_id = thread.id();

        self.threads
            .try_lock()
            .into_lua_err()
            .context("Failed to lock threads vec")?
            .push_back(thread);

        // NOTE: We might be resuming the same thread several times and
        // pushing it to the scheduler several times before it is done,
        // and we should only ever create one result sender per thread
        self.thread_senders
            .try_lock()
            .into_lua_err()
            .context("Failed to lock thread senders vec")?
            .entry(thread_id)
            .or_insert_with(|| SchedulerThreadSender::new(1));

        // NOTE: We might be resuming futures, need to signal that a
        // new lua thread is ready to break out of futures resumption
        self.state.message_sender().send_pushed_lua_thread();

        Ok(thread_id)
    }

    /**
        Waits for the given thread to finish running, and returns its result.
    */
    pub async fn wait_for_thread<'a>(
        &self,
        lua: &'a Lua,
        thread_id: SchedulerThreadId,
    ) -> LuaResult<LuaMultiValue<'a>> {
        let mut recv = {
            let senders = self.thread_senders.lock().await;
            let sender = senders
                .get(&thread_id)
                .expect("Tried to wait for thread that is not queued");
            sender.subscribe()
        };
        let res = match recv.recv().await {
            Err(_) => panic!("Sender was dropped while waiting for {thread_id:?}"),
            Ok(r) => r,
        };
        match res {
            Err(e) => Err(e),
            Ok(k) => {
                let vals = lua
                    .registry_value::<Vec<LuaValue>>(&k)
                    .expect("Received invalid registry key for thread");

                // NOTE: This is not strictly necessary, mlua can clean
                // up registry values on its own, but doing this will add
                // some extra safety and clean up registry values faster
                if let Some(key) = Arc::into_inner(k) {
                    lua.remove_registry_value(key)
                        .expect("Failed to remove registry key for thread");
                }

                Ok(LuaMultiValue::from_vec(vals))
            }
        }
    }
}
