use futures_util::Future;
use mlua::prelude::*;

use super::{traits::IntoLuaThread, SchedulerImpl};

impl<'lua> SchedulerImpl {
    /**
        Schedules a plain future to run whenever the scheduler is available.
    */
    pub fn schedule_future<F>(&self, fut: F)
    where
        F: 'static + Future<Output = ()>,
    {
        self.futures
            .try_lock()
            .expect("Failed to lock futures queue")
            .push(Box::pin(fut))
    }
    /**
        Schedules the given `thread` to run when the given `fut` completes.
    */
    pub fn schedule_thread<T, R, F>(&'lua self, thread: T, fut: F) -> LuaResult<()>
    where
        T: IntoLuaThread<'lua>,
        R: IntoLuaMulti<'lua>,
        F: 'static + Future<Output = LuaResult<R>>,
    {
        let thread = thread.into_lua_thread(&self.lua)?;

        let fut = async move {
            let rets = fut.await.expect("Failed to receive result");
            self.push_back(thread, rets)
                .expect("Failed to schedule future thread");
        };

        // TODO: Lifetime issues
        // self.schedule_future(fut);

        Ok(())
    }
}
