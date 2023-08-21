use futures_util::Future;
use mlua::prelude::*;
use tokio::{
    sync::oneshot::{self, Receiver},
    task,
};

use super::{IntoLuaThread, Scheduler};

impl<'fut> Scheduler<'fut> {
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

        This will potentially spawn the future on a different thread, using
        [`task::spawn`], meaning the provided future must implement [`Send`].

        Returns a [`Receiver`] which may be `await`-ed
        to retrieve the result of the spawned future.

        This [`Receiver`] may be safely ignored if the result of the
        spawned future is not needed, the future will run either way.
    */
    pub fn spawn<F>(&self, fut: F) -> Receiver<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();

        let handle = task::spawn(async move {
            let res = fut.await;
            tx.send(res).ok();
        });

        // NOTE: We must spawn a future on our scheduler which awaits
        // the handle from tokio to start driving our future properly
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
        Equivalent to [`spawn`], except the future is only
        spawned on the Lune scheduler, and on the main thread.
    */
    pub fn spawn_local<F>(&self, fut: F) -> Receiver<F::Output>
    where
        F: Future + 'static,
        F::Output: 'static,
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
    pub fn spawn_thread<F, FR>(
        &'fut self,
        lua: &'fut Lua,
        thread: impl IntoLuaThread<'fut>,
        fut: F,
    ) -> LuaResult<()>
    where
        FR: IntoLuaMulti<'fut>,
        F: Future<Output = LuaResult<FR>> + 'fut,
    {
        let thread = thread.into_lua_thread(lua)?;
        let futs = self.futures_lua.try_lock().expect(
            "Failed to lock futures queue - \
            can't schedule future lua threads during futures resumption",
        );

        futs.push(Box::pin(async move {
            match fut.await.and_then(|rets| rets.into_lua_multi(lua)) {
                Err(e) => {
                    self.push_err(lua, thread, e)
                        .expect("Failed to schedule future err thread");
                }
                Ok(v) => {
                    self.push_back(lua, thread, v)
                        .expect("Failed to schedule future thread");
                }
            }
        }));

        Ok(())
    }
}
