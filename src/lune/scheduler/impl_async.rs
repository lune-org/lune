use futures_util::Future;
use mlua::prelude::*;

use super::{IntoLuaOwnedThread, Scheduler};

impl<'lua, 'fut> Scheduler<'lua, 'fut>
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

        If the given future returns a [`LuaError`], that error will be passed to the given `thread`.
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
        let thread = thread.into_owned_lua_thread(self.lua)?;
        self.schedule_future(async move {
            match fut.await.and_then(|rets| rets.into_lua_multi(self.lua)) {
                Err(e) => {
                    self.push_err(thread, e)
                        .expect("Failed to schedule future err thread");
                }
                Ok(v) => {
                    self.push_back(thread, v)
                        .expect("Failed to schedule future thread");
                }
            }
        });

        Ok(())
    }
}
