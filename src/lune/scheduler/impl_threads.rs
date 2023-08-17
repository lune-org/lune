use mlua::prelude::*;

use super::{thread::SchedulerThread, traits::IntoLuaThread, SchedulerImpl};

impl<'lua> SchedulerImpl {
    /**
        Pops the next thread to run, from the front of the scheduler.

        Returns `None` if there are no threads left to run.
    */
    pub(super) fn pop_thread(
        &'lua self,
    ) -> LuaResult<Option<(LuaThread<'lua>, LuaMultiValue<'lua>)>> {
        match self
            .threads
            .try_borrow_mut()
            .into_lua_err()
            .context("Failed to borrow threads vec")?
            .pop_front()
        {
            Some(thread) => {
                let (thread, args) = thread.into_inner(&self.lua);
                Ok(Some((thread, args)))
            }
            None => Ok(None),
        }
    }

    /**
        Schedules the `thread` to be resumed with the given `args`
        right away, before any other currently scheduled threads.
    */
    pub fn push_front(
        &'lua self,
        thread: impl IntoLuaThread<'lua>,
        args: impl IntoLuaMulti<'lua>,
    ) -> LuaResult<()> {
        let thread = thread.into_lua_thread(&self.lua)?;
        let args = args.into_lua_multi(&self.lua)?;

        self.threads
            .try_borrow_mut()
            .into_lua_err()
            .context("Failed to borrow threads vec")?
            .push_front(SchedulerThread::new(&self.lua, thread, args)?);

        Ok(())
    }

    /**
        Schedules the `thread` to be resumed with the given `args`
        after all other current threads have been resumed.
    */
    pub fn push_back(
        &'lua self,
        thread: impl IntoLuaThread<'lua>,
        args: impl IntoLuaMulti<'lua>,
    ) -> LuaResult<()> {
        let thread = thread.into_lua_thread(&self.lua)?;
        let args = args.into_lua_multi(&self.lua)?;

        self.threads
            .try_borrow_mut()
            .into_lua_err()
            .context("Failed to borrow threads vec")?
            .push_back(SchedulerThread::new(&self.lua, thread, args)?);

        Ok(())
    }
}
