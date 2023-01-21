use std::time::Duration;

use mlua::{Function, UserData, UserDataMethods, Value, Variadic};
use tokio::time;

const DEFAULT_SLEEP_DURATION: f32 = 1.0 / 60.0;

pub struct Task();

impl Task {
    pub fn new() -> Self {
        Self()
    }
}

impl Default for Task {
    fn default() -> Self {
        Self::new()
    }
}

impl UserData for Task {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_function("wait", |_, duration: Option<f32>| async move {
            let secs = duration.unwrap_or(DEFAULT_SLEEP_DURATION);
            time::sleep(Duration::from_secs_f32(secs)).await;
            Ok(secs)
        });
        methods.add_async_function(
            "spawn",
            |lua, (func, args): (Function, Variadic<Value>)| async move {
                let _thread = lua
                    .create_thread(func)?
                    .into_async::<_, Variadic<Value<'lua>>>(args);
                // task::spawn_local(async move { thread });
                Ok(())
            },
        );
    }
}
