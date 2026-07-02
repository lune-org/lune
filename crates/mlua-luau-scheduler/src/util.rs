use futures_lite::StreamExt;
use mlua::prelude::*;
use tracing::instrument;

/**
    Runs a Lua thread until it manually yields (using coroutine.yield), errors, or completes.

    May return `None` if the thread was cancelled.

    Otherwise returns the values yielded by the thread, or the error that caused it to stop.
*/
#[instrument(level = "trace", name = "Scheduler::run_until_yield", skip_all)]
pub(crate) async fn run_until_yield(
    thread: LuaThread,
    args: LuaMultiValue,
) -> Option<LuaResult<LuaMultiValue>> {
    let mut stream = thread.into_async(args).expect("thread must be resumable");
    /*
        NOTE: It is very important that we drop the thread/stream as
        soon as we are done, it takes up valuable Lua registry space
        and detached tasks will not drop until the executor does

        https://github.com/smol-rs/smol/issues/294

        We also do not unwrap here since returning `None` is expected behavior for cancellation.

        Even though we are converting into a stream, and then immediately running it,
        the future may still be cancelled before it is polled, which gives us None.
    */
    stream.next().await
}

/**
    Checks if the given [`LuaValue`] is the async `POLL_PENDING` constant.
*/
#[inline]
pub(crate) fn is_poll_pending(value: &LuaValue) -> bool {
    value
        .as_light_userdata()
        .is_some_and(|l| l == Lua::poll_pending())
}

/**
    Wrapper struct to accept either a Lua thread or a Lua function as function argument.

    [`LuaThreadOrFunction::into_thread`] may be used to convert the value into a Lua thread.
*/
#[derive(Clone)]
pub(crate) enum LuaThreadOrFunction {
    Thread(LuaThread),
    Function(LuaFunction),
}

impl LuaThreadOrFunction {
    pub(super) fn into_thread(self, lua: &Lua) -> LuaResult<LuaThread> {
        match self {
            Self::Thread(t) => Ok(t),
            Self::Function(f) => lua.create_thread(f),
        }
    }
}

impl FromLua for LuaThreadOrFunction {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Thread(t) => Ok(Self::Thread(t)),
            LuaValue::Function(f) => Ok(Self::Function(f)),
            value => Err(LuaError::FromLuaConversionError {
                from: value.type_name(),
                to: "LuaThreadOrFunction".to_string(),
                message: Some("Expected thread or function".to_string()),
            }),
        }
    }
}
