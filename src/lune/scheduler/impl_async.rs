use futures_util::Future;
use mlua::prelude::*;

use super::{traits::IntoLuaOwnedThread, SchedulerImpl};

impl<'lua, 'fut> SchedulerImpl
where
    'lua: 'fut,
{
    /**
        Schedules a plain future to run whenever the scheduler is available.
    */
    pub fn schedule_future<F>(&self, fut: F)
    where
        F: 'static + Future<Output = ()>,
    {
        let futs = self
            .futures
            .try_lock()
            .expect("Failed to lock futures queue");
        futs.push(Box::pin(fut))
    }

    /**
        Schedules the given `thread` to run when the given `fut` completes.
    */
    pub fn schedule_future_thread<T, F, R>(&'lua self, thread: T, fut: F) -> LuaResult<()>
    where
        T: 'static + IntoLuaOwnedThread,
        F: 'static + Future<Output = LuaResult<R>>,
        R: IntoLuaMulti<'fut>,
    {
        let thread = thread.into_owned_lua_thread(&self.lua)?;

        // FIXME: We use self in the future below, so this doesn't compile... how to fix?
        self.schedule_future(async move {
            let rets = fut.await.expect("Failed to receive result");
            self.push_back(thread, rets)
                .expect("Failed to schedule future thread");
        });

        Ok(())
    }
}
