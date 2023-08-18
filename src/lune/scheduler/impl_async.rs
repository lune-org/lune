use futures_util::Future;
use mlua::prelude::*;

use super::Scheduler;

impl<'lua, 'fut> Scheduler<'fut>
where
    'lua: 'fut,
{
    /**
        Schedules a plain future to run whenever the scheduler is available.
    */
    pub fn schedule_future<F>(&'fut self, fut: F)
    where
        F: 'fut + Future<Output = ()>,
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
    pub fn schedule_future_thread<F>(&'fut self, thread: LuaOwnedThread, fut: F) -> LuaResult<()>
    where
        F: 'fut + Future<Output = LuaResult<LuaMultiValue<'fut>>>,
    {
        self.schedule_future(async move {
            let rets = fut.await.expect("Failed to receive result");
            self.push_back(thread, rets)
                .expect("Failed to schedule future thread");
        });

        Ok(())
    }
}
