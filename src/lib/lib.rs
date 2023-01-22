use anyhow::{bail, Result};
use mlua::Lua;

pub mod globals;
pub mod utils;

use crate::{
    globals::{new_console, new_fs, new_net, new_process, new_task},
    utils::formatting::{format_label, pretty_format_luau_error},
};

pub async fn run_lune(name: &str, chunk: &str, args: Vec<String>) -> Result<()> {
    let lua = Lua::new();
    lua.sandbox(true)?;
    // Add in all globals
    {
        let globals = lua.globals();
        globals.raw_set("console", new_console(&lua).await?)?;
        globals.raw_set("fs", new_fs(&lua).await?)?;
        globals.raw_set("net", new_net(&lua).await?)?;
        globals.raw_set("process", new_process(&lua, args.clone()).await?)?;
        globals.raw_set("task", new_task(&lua).await?)?;
        globals.set_readonly(true);
    }
    // Run the requested chunk asynchronously
    let result = lua.load(chunk).set_name(name)?.exec_async().await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => bail!(
            "\n{}\n{}",
            format_label("ERROR"),
            pretty_format_luau_error(&e)
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::run_lune;

    macro_rules! run_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[tokio::test]
                async fn $name() {
                    let args = vec![
                        "Foo".to_owned(),
                        "Bar".to_owned()
                    ];
                    let path = std::env::current_dir()
                        .unwrap()
                        .join(format!("src/tests/{}.luau", $value));
                    let script = tokio::fs::read_to_string(&path)
                        .await
                        .unwrap();
                    if let Err(e) = run_lune($value, &script, args).await {
                        panic!("\nTest '{}' failed!\n{}\n", $value, e.to_string())
                    }
                }
            )*
        }
    }

    run_tests! {
        console_format: "console/format",
        console_set_color: "console/set_color",
        console_set_style: "console/set_style",
        fs_files: "fs/files",
        fs_dirs: "fs/dirs",
        process_args: "process/args",
        process_env: "process/env",
        // NOTE: This test does not currently work, it will exit the entire
        // process, meaning it will also exit our test runner and skip testing
        // process_exit: "process/exit",
        process_spawn: "process/spawn",
        net_request_codes: "net/request/codes",
        net_request_methods: "net/request/methods",
        net_request_redirect: "net/request/redirect",
        net_json_decode: "net/json/decode",
        net_json_encode: "net/json/encode",
        task_cancel: "task/cancel",
        task_defer: "task/defer",
        task_delay: "task/delay",
        task_spawn: "task/spawn",
        task_wait: "task/wait",
    }
}
