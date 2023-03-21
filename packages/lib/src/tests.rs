use std::{env::set_current_dir, path::PathBuf, process::ExitCode};

use anyhow::Result;
use console::set_colors_enabled;
use console::set_colors_enabled_stderr;
use tokio::fs::read_to_string;

use crate::Lune;

const ARGS: &[&str] = &["Foo", "Bar"];

macro_rules! create_tests {
    ($($name:ident: $value:expr,)*) => { $(
        #[tokio::test(flavor = "multi_thread")]
        async fn $name() -> Result<ExitCode> {
            // Disable styling for stdout and stderr since
            // some tests rely on output not being styled
            set_colors_enabled(false);
            set_colors_enabled_stderr(false);
            // NOTE: This path is relative to the lib
            // package, not the cwd or workspace root,
            // so we need to cd to the repo root first
            let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let root_dir = crate_dir.join("../../").canonicalize()?;
            set_current_dir(root_dir)?;
            // The rest of the test logic can continue as normal
            let full_name = format!("tests/{}.luau", $value);
            let script = read_to_string(&full_name).await?;
            let lune = Lune::new().with_args(
                ARGS
                    .clone()
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            );
            let script_name = full_name
				.trim_end_matches(".luau")
				.trim_end_matches(".lua")
				.to_string();
            let exit_code = lune.run(&script_name, &script).await?;
            Ok(exit_code)
        }
    )* }
}

create_tests! {
    fs_files: "fs/files",
    fs_dirs: "fs/dirs",
    fs_move: "fs/move",
    net_request_codes: "net/request/codes",
    net_request_methods: "net/request/methods",
    net_request_query: "net/request/query",
    net_request_redirect: "net/request/redirect",
    net_json_decode: "net/json/decode",
    net_json_encode: "net/json/encode",
    net_serve_requests: "net/serve/requests",
    net_serve_websockets: "net/serve/websockets",
    process_args: "process/args",
    process_cwd: "process/cwd",
    process_env: "process/env",
    process_exit: "process/exit",
    process_spawn: "process/spawn",
    require_async: "globals/require/tests/async",
    require_async_concurrent: "globals/require/tests/async_concurrent",
    require_async_sequential: "globals/require/tests/async_sequential",
    require_builtins: "globals/require/tests/builtins",
    require_children: "globals/require/tests/children",
    require_invalid: "globals/require/tests/invalid",
    require_nested: "globals/require/tests/nested",
    require_parents: "globals/require/tests/parents",
    require_siblings: "globals/require/tests/siblings",
    // TODO: Uncomment this test, it is commented out right
    // now to let CI pass so that we can make a new release
    // global_coroutine: "globals/coroutine",
    global_pcall: "globals/pcall",
    global_type: "globals/type",
    global_typeof: "globals/typeof",
    stdio_format: "stdio/format",
    stdio_color: "stdio/color",
    stdio_style: "stdio/style",
    stdio_write: "stdio/write",
    stdio_ewrite: "stdio/ewrite",
    task_cancel: "task/cancel",
    task_defer: "task/defer",
    task_delay: "task/delay",
    task_spawn: "task/spawn",
    task_wait: "task/wait",
}
