use std::time::Duration;

use mlua::{Lua, Result};
use tokio::time;

use crate::utils::table_builder::ReadonlyTableBuilder;

const DEFAULT_SLEEP_DURATION: f32 = 1.0 / 60.0;

pub async fn create(lua: Lua) -> Result<Lua> {
    lua.globals().raw_set(
        "task",
        ReadonlyTableBuilder::new(&lua)?
            .with_async_function("wait", task_wait)?
            .build()?,
    )?;
    Ok(lua)
}

// FIXME: It does seem possible to properly make an async wait
// function with mlua right now, something breaks when using
// async wait functions inside of coroutines
async fn task_wait(_: &Lua, duration: Option<f32>) -> Result<f32> {
    let secs = duration.unwrap_or(DEFAULT_SLEEP_DURATION);
    time::sleep(Duration::from_secs_f32(secs)).await;
    Ok(secs)
}
