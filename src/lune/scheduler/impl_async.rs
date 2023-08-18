use futures_util::Future;
use mlua::prelude::*;

use super::{IntoLuaOwnedThread, Scheduler};

impl<'lua, 'fut> Scheduler<'fut>
where
    'lua: 'fut,
{
    /**
        Schedules a plain future to run whenever the scheduler is available.
    */
    pub fn schedule_future<F>(&'fut self, fut: F)
    where
        F: Future<Output = ()> + 'fut,
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
    pub fn schedule_future_thread<F, FR>(
        &'fut self,
        thread: impl IntoLuaOwnedThread,
        fut: F,
    ) -> LuaResult<()>
    where
        FR: IntoLuaMulti<'fut>,
        F: Future<Output = LuaResult<FR>> + 'fut,
    {
        let thread = thread.into_owned_lua_thread(&self.lua)?;
        self.schedule_future(async move {
            let rets = fut.await.expect("Failed to receive result");
            let rets = rets
                .into_lua_multi(&self.lua)
                .expect("Failed to create return multi value");
            self.push_back(thread, rets)
                .expect("Failed to schedule future thread");
        });

        Ok(())
    }
}
