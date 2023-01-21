use std::time::Duration;

use mlua::{Function, Lua, Result, Table, Value, Variadic};
use tokio::time;

const DEFAULT_SLEEP_DURATION: f32 = 1.0 / 60.0;

pub fn new(lua: &Lua) -> Result<Table> {
    let tab = lua.create_table()?;
    tab.raw_set(
        "defer",
        lua.create_async_function(
            |lua, (func, args): (Function, Variadic<Value>)| async move {
                let thread = lua.create_thread(func)?;
                thread.into_async(args).await?;
                Ok(())
            },
        )?,
    )?;
    tab.raw_set(
        "delay",
        lua.create_async_function(
            |lua, (func, duration, args): (Function, Option<f32>, Variadic<Value>)| async move {
                let secs = duration.unwrap_or(DEFAULT_SLEEP_DURATION);
                time::sleep(Duration::from_secs_f32(secs)).await;
                let thread = lua.create_thread(func)?;
                thread.into_async(args).await?;
                Ok(())
            },
        )?,
    )?;
    tab.raw_set(
        "spawn",
        lua.create_async_function(
            |lua, (func, args): (Function, Variadic<Value>)| async move {
                let thread = lua.create_thread(func)?;
                thread.into_async(args).await?;
                Ok(())
            },
        )?,
    )?;
    tab.raw_set(
        "wait",
        lua.create_async_function(|_, duration: Option<f32>| async move {
            let secs = duration.unwrap_or(DEFAULT_SLEEP_DURATION);
            time::sleep(Duration::from_secs_f32(secs)).await;
            Ok(secs)
        })?,
    )?;
    tab.set_readonly(true);
    Ok(tab)
}
