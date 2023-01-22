use std::time::Duration;

use mlua::{Error, Function, Lua, Result, Table, Thread, Value, Variadic};
use tokio::time::{self, Instant};

use crate::utils::table_builder::ReadonlyTableBuilder;

pub async fn create(lua: &Lua) -> Result<()> {
    lua.globals().raw_set(
        "task",
        ReadonlyTableBuilder::new(lua)?
            .with_async_function("cancel", task_cancel)?
            .with_async_function("defer", task_defer)?
            .with_async_function("delay", task_delay)?
            .with_async_function("spawn", task_spawn)?
            .with_async_function("wait", task_wait)?
            .build()?,
    )
}

fn get_thread_from_arg<'a>(lua: &'a Lua, thread_or_function_arg: Value<'a>) -> Result<Thread<'a>> {
    Ok(match thread_or_function_arg {
        Value::Thread(thread) => thread,
        Value::Function(func) => lua.create_thread(func)?,
        val => {
            return Err(Error::RuntimeError(format!(
                "Expected type thread or function, got {}",
                val.type_name()
            )))
        }
    })
}

async fn task_cancel(lua: &Lua, thread: Thread<'_>) -> Result<()> {
    let coroutine: Table = lua.globals().raw_get("coroutine")?;
    let close: Function = coroutine.raw_get("close")?;
    close.call_async(thread).await?;
    Ok(())
}

async fn task_defer<'a>(lua: &Lua, (tof, args): (Value<'a>, Variadic<Value<'a>>)) -> Result<()> {
    task_wait(lua, None).await?;
    get_thread_from_arg(lua, tof)?
        .into_async::<_, Variadic<Value<'_>>>(args)
        .await?;
    Ok(())
}

async fn task_delay<'a>(
    lua: &Lua,
    (delay, tof, args): (Option<f32>, Value<'a>, Variadic<Value<'a>>),
) -> Result<()> {
    task_wait(lua, delay).await?;
    get_thread_from_arg(lua, tof)?
        .into_async::<_, Variadic<Value<'_>>>(args)
        .await?;
    Ok(())
}

async fn task_spawn<'a>(lua: &Lua, (tof, args): (Value<'a>, Variadic<Value<'a>>)) -> Result<()> {
    get_thread_from_arg(lua, tof)?
        .into_async::<_, Variadic<Value<'_>>>(args)
        .await?;
    Ok(())
}

// FIXME: It doesn't seem possible to properly make an async wait
// function with mlua right now, something breaks when using
// the async wait function inside of a coroutine
async fn task_wait(_: &Lua, duration: Option<f32>) -> Result<f32> {
    let start = Instant::now();
    time::sleep(
        duration
            .map(Duration::from_secs_f32)
            .unwrap_or(Duration::ZERO),
    )
    .await;
    let end = Instant::now();
    Ok((end - start).as_secs_f32())
}
