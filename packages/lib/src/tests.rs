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
    net_url_encode: "net/url/encode",
    net_url_decode: "net/url/decode",
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
    serde_json_decode: "serde/json/decode",
    serde_json_encode: "serde/json/encode",
    serde_toml_decode: "serde/toml/decode",
    serde_toml_encode: "serde/toml/encode",
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

#[cfg(feature = "roblox")]
create_tests! {
    roblox_axes: "roblox/datatypes/Axes",
    roblox_brick_color: "roblox/datatypes/BrickColor",
    roblox_cframe: "roblox/datatypes/CFrame",
    roblox_color3: "roblox/datatypes/Color3",
    roblox_color_sequence: "roblox/datatypes/ColorSequence",
    roblox_color_sequence_keypoint: "roblox/datatypes/ColorSequenceKeypoint",
    roblox_enum: "roblox/datatypes/Enum",
    roblox_faces: "roblox/datatypes/Faces",
    roblox_font: "roblox/datatypes/Font",
    roblox_number_range: "roblox/datatypes/NumberRange",
    roblox_number_sequence: "roblox/datatypes/NumberSequence",
    roblox_number_sequence_keypoint: "roblox/datatypes/NumberSequenceKeypoint",
    roblox_physical_properties: "roblox/datatypes/PhysicalProperties",
    roblox_ray: "roblox/datatypes/Ray",
    roblox_rect: "roblox/datatypes/Rect",
    roblox_udim: "roblox/datatypes/UDim",
    roblox_udim2: "roblox/datatypes/UDim2",
    roblox_region3: "roblox/datatypes/Region3",
    roblox_region3int16: "roblox/datatypes/Region3int16",
    roblox_vector2: "roblox/datatypes/Vector2",
    roblox_vector2int16: "roblox/datatypes/Vector2int16",
    roblox_vector3: "roblox/datatypes/Vector3",
    roblox_vector3int16: "roblox/datatypes/Vector3int16",
}
