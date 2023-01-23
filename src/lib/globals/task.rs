use std::time::{Duration, Instant};

use mlua::prelude::*;
use smol::Timer;

use crate::utils::table_builder::TableBuilder;

const TASK_LIB: &str = include_str!("../luau/task.luau");

pub async fn create(lua: &Lua) -> LuaResult<()> {
    let wait = lua.create_async_function(move |_, duration: Option<f32>| async move {
        let start = Instant::now();
        Timer::after(
            duration
                .map(Duration::from_secs_f32)
                .unwrap_or(Duration::ZERO),
        )
        .await;
        let end = Instant::now();
        Ok((end - start).as_secs_f32())
    })?;
    let task_lib: LuaTable = lua
        .load(TASK_LIB)
        .set_name("task")?
        .call_async(wait.clone())
        .await?;
    lua.globals().raw_set(
        "task",
        TableBuilder::new(lua)?
            .with_value("cancel", task_lib.raw_get::<_, LuaFunction>("cancel")?)?
            .with_value("defer", task_lib.raw_get::<_, LuaFunction>("defer")?)?
            .with_value("delay", task_lib.raw_get::<_, LuaFunction>("delay")?)?
            .with_value("spawn", task_lib.raw_get::<_, LuaFunction>("spawn")?)?
            .with_value("wait", wait)?
            .build_readonly()?,
    )
}
