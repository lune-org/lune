use std::time::Duration;

use mlua::{Function, Lua, Result, Table, Value, Variadic};
use tokio::time;

use crate::utils::table_builder::ReadonlyTableBuilder;

const DEFAULT_SLEEP_DURATION: f32 = 1.0 / 60.0;

pub fn new(lua: &Lua) -> Result<Table> {
    ReadonlyTableBuilder::new(lua)?
        .with_async_function(
            "defer",
            |lua, (func, args): (Function, Variadic<Value>)| async move {
                let thread = lua.create_thread(func)?;
                thread.into_async(args).await?;
                Ok(())
            },
        )?
        .with_async_function(
            "delay",
            |lua, (func, duration, args): (Function, Option<f32>, Variadic<Value>)| async move {
                let secs = duration.unwrap_or(DEFAULT_SLEEP_DURATION);
                time::sleep(Duration::from_secs_f32(secs)).await;
                let thread = lua.create_thread(func)?;
                thread.into_async(args).await?;
                Ok(())
            },
        )?
        .with_async_function(
            "spawn",
            |lua, (func, args): (Function, Variadic<Value>)| async move {
                let thread = lua.create_thread(func)?;
                thread.into_async(args).await?;
                Ok(())
            },
        )?
        .with_async_function("wait", |_, duration: Option<f32>| async move {
            let secs = duration.unwrap_or(DEFAULT_SLEEP_DURATION);
            time::sleep(Duration::from_secs_f32(secs)).await;
            Ok(secs)
        })?
        .build()
}
