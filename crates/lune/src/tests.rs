use std::env::set_current_dir;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use console::set_colors_enabled;
use console::set_colors_enabled_stderr;

use lune_utils::path::clean_path;

use crate::Runtime;

const ARGS: &[&str] = &["Foo", "Bar"];

fn run_test(path: &str) -> Result<ExitCode> {
    async_io::block_on(async {
        // We need to change the current directory to the workspace root since
        // we are in a sub-crate and tests would run relative to the sub-crate
        let workspace_dir_str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");
        let workspace_dir = clean_path(PathBuf::from(workspace_dir_str));
        set_current_dir(&workspace_dir)?;

        // Disable styling for stdout and stderr since
        // some tests rely on output not being styled
        set_colors_enabled(false);
        set_colors_enabled_stderr(false);

        // The rest of the test logic can continue as normal
        let mut rt = Runtime::new()?.with_args(ARGS).with_jit(true);

        let script_path = workspace_dir.join("tests").join(format!("{path}.luau"));
        let script_values = rt.run_file(script_path).await?;

        Ok(ExitCode::from(script_values.status()))
    })
}

macro_rules! create_tests {
    ($($name:ident: $value:expr,)*) => { $(
        #[test]
        fn $name() -> Result<ExitCode> {
        	run_test($value)
        }
    )* }
}

#[cfg(any(
    feature = "std-datetime",
    feature = "std-fs",
    feature = "std-luau",
    feature = "std-net",
    feature = "std-process",
    feature = "std-regex",
    feature = "std-roblox",
    feature = "std-serde",
    feature = "std-stdio",
    feature = "std-task",
))]
create_tests! {
    require_aliases: "require/tests/aliases",
    require_async: "require/tests/async",
    require_async_concurrent: "require/tests/async_concurrent",
    require_async_sequential: "require/tests/async_sequential",
    require_builtins: "require/tests/builtins",
    require_children: "require/tests/children",
    require_init: "require/tests/init_files",
    require_invalid: "require/tests/invalid",
    require_multi_ext: "require/tests/multi_ext",
    require_nested: "require/tests/nested",
    require_parents: "require/tests/parents",
    require_siblings: "require/tests/siblings",
    require_state: "require/tests/state",

    global_g_table: "globals/_G",
    global_version: "globals/_VERSION",
    global_coroutine: "globals/coroutine",
    global_error: "globals/error",
    global_pcall: "globals/pcall",
    global_type: "globals/type",
    global_typeof: "globals/typeof",
    global_warn: "globals/warn",
}

#[cfg(feature = "std-datetime")]
create_tests! {
    datetime_format_local_time: "datetime/formatLocalTime",
    datetime_format_universal_time: "datetime/formatUniversalTime",
    datetime_from_rfc_2822: "datetime/fromRfc2822",
    datetime_from_rfc_3339: "datetime/fromRfc3339",
    datetime_from_local_time: "datetime/fromLocalTime",
    datetime_from_universal_time: "datetime/fromUniversalTime",
    datetime_from_unix_timestamp: "datetime/fromUnixTimestamp",
    datetime_now: "datetime/now",
    datetime_to_rfc_2822: "datetime/toRfc2822",
    datetime_to_rfc_3339: "datetime/toRfc3339",
    datetime_to_local_time: "datetime/toLocalTime",
    datetime_to_universal_time: "datetime/toUniversalTime",
}

#[cfg(feature = "std-fs")]
create_tests! {
    fs_files: "fs/files",
    fs_copy: "fs/copy",
    fs_dirs: "fs/dirs",
    fs_metadata: "fs/metadata",
    fs_move: "fs/move",
}

#[cfg(feature = "std-luau")]
create_tests! {
    luau_compile: "luau/compile",
    luau_load: "luau/load",
    luau_options: "luau/options",
    luau_safeenv: "luau/safeenv",
}

#[cfg(feature = "std-net")]
create_tests! {
    net_request_codes: "net/request/codes",
    net_request_compression: "net/request/compression",
    net_request_https: "net/request/https",
    net_request_methods: "net/request/methods",
    net_request_query: "net/request/query",
    net_request_redirect: "net/request/redirect",

    net_serve_addresses: "net/serve/addresses",
    net_serve_handles: "net/serve/handles",
    net_serve_non_blocking: "net/serve/non_blocking",
    net_serve_requests: "net/serve/requests",
    net_serve_websockets: "net/serve/websockets",

    net_socket_basic: "net/socket/basic",
    net_socket_wss: "net/socket/wss",
    net_socket_wss_rw: "net/socket/wss_rw",

    net_tcp_basic: "net/tcp/basic",
    net_tcp_info: "net/tcp/info",
    net_tcp_tls: "net/tcp/tls",

    net_url_encode: "net/url/encode",
    net_url_decode: "net/url/decode",
}

#[cfg(feature = "std-process")]
create_tests! {
    process_args: "process/args",
    process_cwd: "process/cwd",
    process_env: "process/env",
    process_exit: "process/exit",
    process_exec_async: "process/exec/async",
    process_exec_basic: "process/exec/basic",
    process_exec_cwd: "process/exec/cwd",
    process_exec_no_panic: "process/exec/no_panic",
    process_exec_shell: "process/exec/shell",
    process_exec_stdin: "process/exec/stdin",
    process_exec_stdio: "process/exec/stdio",
    process_spawn_non_blocking: "process/create/non_blocking",
    process_spawn_status: "process/create/status",
    process_spawn_stream: "process/create/stream",
}

#[cfg(feature = "std-regex")]
create_tests! {
    regex_general: "regex/general",
    regex_metamethods: "regex/metamethods",
    regex_replace: "regex/replace",
}

#[cfg(feature = "std-roblox")]
create_tests! {
    roblox_datatype_axes: "roblox/datatypes/Axes",
    roblox_datatype_brick_color: "roblox/datatypes/BrickColor",
    roblox_datatype_cframe: "roblox/datatypes/CFrame",
    roblox_datatype_color3: "roblox/datatypes/Color3",
    roblox_datatype_color_sequence: "roblox/datatypes/ColorSequence",
    roblox_datatype_color_sequence_keypoint: "roblox/datatypes/ColorSequenceKeypoint",
    roblox_datatype_content: "roblox/datatypes/Content",
    roblox_datatype_enum: "roblox/datatypes/Enum",
    roblox_datatype_faces: "roblox/datatypes/Faces",
    roblox_datatype_font: "roblox/datatypes/Font",
    roblox_datatype_number_range: "roblox/datatypes/NumberRange",
    roblox_datatype_number_sequence: "roblox/datatypes/NumberSequence",
    roblox_datatype_number_sequence_keypoint: "roblox/datatypes/NumberSequenceKeypoint",
    roblox_datatype_physical_properties: "roblox/datatypes/PhysicalProperties",
    roblox_datatype_ray: "roblox/datatypes/Ray",
    roblox_datatype_rect: "roblox/datatypes/Rect",
    roblox_datatype_udim: "roblox/datatypes/UDim",
    roblox_datatype_udim2: "roblox/datatypes/UDim2",
    roblox_datatype_region3: "roblox/datatypes/Region3",
    roblox_datatype_region3int16: "roblox/datatypes/Region3int16",
    roblox_datatype_vector2: "roblox/datatypes/Vector2",
    roblox_datatype_vector2int16: "roblox/datatypes/Vector2int16",
    roblox_datatype_vector3: "roblox/datatypes/Vector3",
    roblox_datatype_vector3int16: "roblox/datatypes/Vector3int16",

    roblox_files_deserialize_model: "roblox/files/deserializeModel",
    roblox_files_deserialize_place: "roblox/files/deserializePlace",
    roblox_files_serialize_model: "roblox/files/serializeModel",
    roblox_files_serialize_place: "roblox/files/serializePlace",

    roblox_instance_attributes: "roblox/instance/attributes",
    roblox_instance_new: "roblox/instance/new",
    roblox_instance_properties: "roblox/instance/properties",
    roblox_instance_tags: "roblox/instance/tags",

    roblox_instance_classes_data_model: "roblox/instance/classes/DataModel",
    roblox_instance_classes_workspace: "roblox/instance/classes/Workspace",
    roblox_instance_classes_terrain: "roblox/instance/classes/Terrain",

    roblox_instance_custom_async: "roblox/instance/custom/async",
    roblox_instance_custom_methods: "roblox/instance/custom/methods",
    roblox_instance_custom_properties: "roblox/instance/custom/properties",

    roblox_instance_methods_clear_all_children: "roblox/instance/methods/ClearAllChildren",
    roblox_instance_methods_clone: "roblox/instance/methods/Clone",
    roblox_instance_methods_destroy: "roblox/instance/methods/Destroy",
    roblox_instance_methods_find_first_ancestor: "roblox/instance/methods/FindFirstAncestor",
    roblox_instance_methods_find_first_ancestor_of_class: "roblox/instance/methods/FindFirstAncestorOfClass",
    roblox_instance_methods_find_first_ancestor_which_is_a: "roblox/instance/methods/FindFirstAncestorWhichIsA",
    roblox_instance_methods_find_first_child: "roblox/instance/methods/FindFirstChild",
    roblox_instance_methods_find_first_child_of_class: "roblox/instance/methods/FindFirstChildOfClass",
    roblox_instance_methods_find_first_child_which_is_a: "roblox/instance/methods/FindFirstChildWhichIsA",
    roblox_instance_methods_get_children: "roblox/instance/methods/GetChildren",
    roblox_instance_methods_get_debug_id: "roblox/instance/methods/GetDebugId",
    roblox_instance_methods_get_descendants: "roblox/instance/methods/GetDescendants",
    roblox_instance_methods_get_full_name: "roblox/instance/methods/GetFullName",
    roblox_instance_methods_is_a: "roblox/instance/methods/IsA",
    roblox_instance_methods_is_ancestor_of: "roblox/instance/methods/IsAncestorOf",
    roblox_instance_methods_is_descendant_of: "roblox/instance/methods/IsDescendantOf",

    roblox_misc_typeof: "roblox/misc/typeof",

    roblox_reflection_class: "roblox/reflection/class",
    roblox_reflection_database: "roblox/reflection/database",
    roblox_reflection_enums: "roblox/reflection/enums",
    roblox_reflection_property: "roblox/reflection/property",
}

#[cfg(feature = "std-serde")]
create_tests! {
    serde_compression_files: "serde/compression/files",
    serde_compression_roundtrip: "serde/compression/roundtrip",
    serde_json_decode: "serde/json/decode",
    serde_json_encode: "serde/json/encode",
    serde_toml_decode: "serde/toml/decode",
    serde_toml_encode: "serde/toml/encode",
    serde_hashing_hash: "serde/hashing/hash",
    serde_hashing_hmac: "serde/hashing/hmac",
}

#[cfg(feature = "std-stdio")]
create_tests! {
    stdio_format: "stdio/format",
    stdio_color: "stdio/color",
    stdio_style: "stdio/style",
    stdio_write: "stdio/write",
    stdio_ewrite: "stdio/ewrite",
}

#[cfg(feature = "std-task")]
create_tests! {
    task_cancel: "task/cancel",
    task_defer: "task/defer",
    task_delay: "task/delay",
    task_spawn: "task/spawn",
    task_wait: "task/wait",
}
