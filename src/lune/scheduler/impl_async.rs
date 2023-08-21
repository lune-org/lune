use futures_util::Future;
use mlua::prelude::*;
use tokio::{
    sync::oneshot::{self, Receiver},
    task,
};

use super::{IntoLuaThread, Scheduler};

impl<'lua, 'fut> Scheduler<'lua, 'fut>
where
    'lua: 'fut,
{
    /**
        Checks if there are any futures to run, for
        lua futures and background futures respectively.
    */
    pub(super) fn has_futures(&self) -> (bool, bool) {
        (
            self.futures_lua
                .try_lock()
                .expect("Failed to lock lua futures for check")
                .len()
                > 0,
            self.futures_background
                .try_lock()
                .expect("Failed to lock background futures for check")
                .len()
                > 0,
        )
    }

    /**
        Schedules a plain future to run in the background.

        This will spawn the future both on the scheduler and
        potentially on a different thread using [`task::spawn`],
        meaning the provided future must implement [`Send`].

        Returns a [`Receiver`] which may be `await`-ed
        to retrieve the result of the spawned future.

        This [`Receiver`] may be safely ignored if the result of the
        spawned future is not needed, the future will run either way.
    */
    pub fn schedule_future_background<F, FR>(&self, fut: F) -> Receiver<FR>
    where
        F: Future<Output = FR> + Send + 'static,
        FR: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        let handle = task::spawn(async move {
            let res = fut.await;
            tx.send(res).ok();
        });

        let futs = self
            .futures_background
            .try_lock()
            .expect("Failed to lock futures queue for background tasks");
        futs.push(Box::pin(async move {
            handle.await.ok();
        }));

        rx
    }

    /**
        Equivalent to [`schedule_future_background`], except the
        future is only spawned on the scheduler, on the main thread.
    */
    pub fn schedule_future_background_local<F, FR>(&self, fut: F) -> Receiver<FR>
    where
        F: Future<Output = FR> + 'static,
        FR: 'static,
    {
        let (tx, rx) = oneshot::channel();

        let futs = self
            .futures_background
            .try_lock()
            .expect("Failed to lock futures queue for background tasks");
        futs.push(Box::pin(async move {
            let res = fut.await;
            tx.send(res).ok();
        }));

        rx
    }

    /**
        Schedules the given `thread` to run when the given `fut` completes.

        If the given future returns a [`LuaError`], that error will be passed to the given `thread`.
    */
    pub fn schedule_future_thread<F, FR>(
        &'fut self,
        thread: impl IntoLuaThread<'fut>,
        fut: F,
    ) -> LuaResult<()>
    where
        FR: IntoLuaMulti<'fut>,
        F: Future<Output = LuaResult<FR>> + 'fut,
    {
        let thread = thread.into_lua_thread(self.lua)?;
        let futs = self.futures_lua.try_lock().expect(
            "Failed to lock futures queue - \
            can't schedule future lua threads during futures resumption",
        );

        futs.push(Box::pin(async move {
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
        }));

        Ok(())
    }
}
