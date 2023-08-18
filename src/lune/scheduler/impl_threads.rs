use std::sync::Arc;

use mlua::prelude::*;

use super::{
    thread::{SchedulerThread, SchedulerThreadId, SchedulerThreadSender},
    Scheduler,
};

impl<'lua, 'fut> Scheduler<'fut>
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
    pub(super) fn pop_thread(
        &self,
    ) -> LuaResult<Option<(LuaOwnedThread, LuaMultiValue<'_>, SchedulerThreadSender)>> {
        match self
            .threads
            .try_borrow_mut()
            .into_lua_err()
            .context("Failed to borrow threads vec")?
            .pop_front()
        {
            Some(thread) => {
                let thread_id = &thread.id();
                let (thread, args) = thread.into_inner(&self.lua);
                let sender = self
                    .thread_senders
                    .borrow_mut()
                    .remove(thread_id)
                    .expect("Missing thread sender");
                Ok(Some((thread, args, sender)))
            }
            None => Ok(None),
        }
    }

    /**
        Schedules the `thread` to be resumed with the given `args`
        right away, before any other currently scheduled threads.
    */
    pub fn push_front(
        &self,
        thread: LuaOwnedThread,
        args: LuaMultiValue<'_>,
    ) -> LuaResult<SchedulerThreadId> {
        let thread = SchedulerThread::new(&self.lua, thread, args)?;
        let thread_id = thread.id();

        self.threads
            .try_borrow_mut()
            .into_lua_err()
            .context("Failed to borrow threads vec")?
            .push_front(thread);
        self.thread_senders
            .borrow_mut()
            .insert(thread_id, SchedulerThreadSender::new(1));

        Ok(thread_id)
    }

    /**
        Schedules the `thread` to be resumed with the given `args`
        after all other current threads have been resumed.
    */
    pub fn push_back(
        &self,
        thread: LuaOwnedThread,
        args: LuaMultiValue<'_>,
    ) -> LuaResult<SchedulerThreadId> {
        let thread = SchedulerThread::new(&self.lua, thread, args)?;
        let thread_id = thread.id();

        self.threads
            .try_borrow_mut()
            .into_lua_err()
            .context("Failed to borrow threads vec")?
            .push_back(thread);
        self.thread_senders
            .borrow_mut()
            .insert(thread_id, SchedulerThreadSender::new(1));

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
        match recv.recv().await.expect("Failed to receive thread result") {
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
