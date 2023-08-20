use std::sync::Arc;

use mlua::prelude::*;

use super::{
    thread::{SchedulerThread, SchedulerThreadId, SchedulerThreadSender},
    IntoLuaOwnedThread, Scheduler,
};

impl<'lua, 'fut> Scheduler<'lua, 'fut>
where
    'lua: 'fut,
{
    /**
        Checks if there are any lua threads to run.
    */
    pub(super) fn has_thread(&self) -> bool {
        !self
            .threads
            .try_borrow()
            .expect("Failed to borrow threads vec")
            .is_empty()
    }

    /**
        Pops the next thread to run, from the front of the scheduler.

        Returns `None` if there are no threads left to run.
    */
    pub(super) fn pop_thread(&self) -> LuaResult<Option<SchedulerThread>> {
        match self
            .threads
            .try_borrow_mut()
            .into_lua_err()
            .context("Failed to borrow threads vec")?
            .pop_front()
        {
            Some(thread) => Ok(Some(thread)),
            None => Ok(None),
        }
    }

    /**
        Schedules the `thread` to be resumed with the given [`LuaError`].
    */
    pub fn push_err(&self, thread: impl IntoLuaOwnedThread, err: LuaError) -> LuaResult<()> {
        let thread = thread.into_owned_lua_thread(self.lua)?;
        let args = LuaMultiValue::new(); // Will be resumed with error, don't need real args

        let thread = SchedulerThread::new(self.lua, thread, args)?;
        let thread_id = thread.id();

        self.state.set_thread_error(thread_id, err);
        self.threads
            .try_borrow_mut()
            .into_lua_err()
            .context("Failed to borrow threads vec")?
            .push_front(thread);

        // NOTE: We might be resuming futures, need to signal that a
        // new lua thread is ready to break out of futures resumption
        if self.new_thread_ready.receiver_count() > 0 {
            self.new_thread_ready.send(()).ok();
        }

        Ok(())
    }

    /**
        Schedules the `thread` to be resumed with the given `args`
        right away, before any other currently scheduled threads.
    */
    pub fn push_front<'a>(
        &'a self,
        thread: impl IntoLuaOwnedThread,
        args: impl IntoLuaMulti<'a>,
    ) -> LuaResult<SchedulerThreadId> {
        let thread = thread.into_owned_lua_thread(self.lua)?;
        let args = args.into_lua_multi(self.lua)?;

        let thread = SchedulerThread::new(self.lua, thread, args)?;
        let thread_id = thread.id();

        self.threads
            .try_borrow_mut()
            .into_lua_err()
            .context("Failed to borrow threads vec")?
            .push_front(thread);

        // NOTE: We might be resuming the same thread several times and
        // pushing it to the scheduler several times before it is done,
        // and we should only ever create one result sender per thread
        self.thread_senders
            .borrow_mut()
            .entry(thread_id)
            .or_insert_with(|| SchedulerThreadSender::new(1));

        // NOTE: We might be resuming futures, need to signal that a
        // new lua thread is ready to break out of futures resumption
        if self.new_thread_ready.receiver_count() > 0 {
            self.new_thread_ready.send(()).ok();
        }

        Ok(thread_id)
    }

    /**
        Schedules the `thread` to be resumed with the given `args`
        after all other current threads have been resumed.
    */
    pub fn push_back<'a>(
        &'a self,
        thread: impl IntoLuaOwnedThread,
        args: impl IntoLuaMulti<'a>,
    ) -> LuaResult<SchedulerThreadId> {
        let thread = thread.into_owned_lua_thread(self.lua)?;
        let args = args.into_lua_multi(self.lua)?;

        let thread = SchedulerThread::new(self.lua, thread, args)?;
        let thread_id = thread.id();

        self.threads
            .try_borrow_mut()
            .into_lua_err()
            .context("Failed to borrow threads vec")?
            .push_back(thread);

        // NOTE: We might be resuming the same thread several times and
        // pushing it to the scheduler several times before it is done,
        // and we should only ever create one result sender per thread
        self.thread_senders
            .borrow_mut()
            .entry(thread_id)
            .or_insert_with(|| SchedulerThreadSender::new(1));

        // NOTE: We might be resuming futures, need to signal that a
        // new lua thread is ready to break out of futures resumption
        if self.new_thread_ready.receiver_count() > 0 {
            self.new_thread_ready.send(()).ok();
        }

        Ok(thread_id)
    }

    /**
        Waits for the given thread to finish running, and returns its result.
    */
    pub async fn wait_for_thread(
        &self,
        thread_id: SchedulerThreadId,
    ) -> LuaResult<LuaMultiValue<'_>> {
        let mut recv = {
            let senders = self.thread_senders.borrow();
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
                let vals = self
                    .lua
                    .registry_value::<Vec<LuaValue>>(&k)
                    .expect("Received invalid registry key for thread");

                // NOTE: This is not strictly necessary, mlua can clean
                // up registry values on its own, but doing this will add
                // some extra safety and clean up registry values faster
                if let Some(key) = Arc::into_inner(k) {
                    self.lua
                        .remove_registry_value(key)
                        .expect("Failed to remove registry key for thread");
                }

                Ok(LuaMultiValue::from_vec(vals))
            }
        }
    }
}
