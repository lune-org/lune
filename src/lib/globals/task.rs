use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use mlua::{Function, Lua, Result, Table, Thread, Value, Variadic};
use tokio::time;

use crate::utils::table_builder::ReadonlyTableBuilder;

const DEFAULT_SLEEP_DURATION: f32 = 1.0 / 60.0;

#[allow(dead_code)]
pub struct WaitingThread<'a> {
    is_delayed_for: Option<f32>,
    is_deferred: Option<bool>,
    thread: Thread<'a>,
    args: Variadic<Value<'a>>,
}

pub fn new<'a>(lua: &'a Lua, _threads: &Arc<Mutex<Vec<WaitingThread<'a>>>>) -> Result<Table<'a>> {
    // TODO: Figure out how to insert into threads vec
    ReadonlyTableBuilder::new(lua)?
        .with_function("cancel", |lua, thread: Thread| {
            thread.reset(lua.create_function(|_, _: ()| Ok(()))?)?;
            Ok(())
        })?
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
            |lua, (duration, func, args): (Option<f32>, Function, Variadic<Value>)| async move {
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
