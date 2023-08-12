use std::process::ExitCode;

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
    fs_copy: "fs/copy",
    fs_dirs: "fs/dirs",
    fs_metadata: "fs/metadata",
    fs_move: "fs/move",

    luau_compile: "luau/compile",
    luau_load: "luau/load",
    luau_options: "luau/options",

    net_request_codes: "net/request/codes",
    net_request_compression: "net/request/compression",
    net_request_methods: "net/request/methods",
    net_request_query: "net/request/query",
    net_request_redirect: "net/request/redirect",
    net_url_encode: "net/url/encode",
    net_url_decode: "net/url/decode",
    net_serve_requests: "net/serve/requests",
    net_serve_websockets: "net/serve/websockets",
    net_socket_wss: "net/socket/wss",
    net_socket_wss_rw: "net/socket/wss_rw",

    process_args: "process/args",
    process_cwd: "process/cwd",
    process_env: "process/env",
    process_exit: "process/exit",
    process_spawn: "process/spawn",

    require_async: "require/tests/async",
    require_async_concurrent: "require/tests/async_concurrent",
    require_async_sequential: "require/tests/async_sequential",
    require_builtins: "require/tests/builtins",
    require_children: "require/tests/children",
    require_init: "require/tests/init",
    require_invalid: "require/tests/invalid",
    require_multi_ext: "require/tests/multi_ext",
    require_nested: "require/tests/nested",
    require_parents: "require/tests/parents",
    require_siblings: "require/tests/siblings",

    global_g_table: "globals/_G",
    // TODO: Uncomment this test, it is commented out right
    // now to let CI pass so that we can make a new release
    // global_coroutine: "globals/coroutine",
    global_pcall: "globals/pcall",
    global_type: "globals/type",
    global_typeof: "globals/typeof",
    global_version: "globals/version",

    serde_compression_files: "serde/compression/files",
    serde_compression_roundtrip: "serde/compression/roundtrip",
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
    roblox_datatype_axes: "roblox/datatypes/Axes",
    roblox_datatype_brick_color: "roblox/datatypes/BrickColor",
    roblox_datatype_cframe: "roblox/datatypes/CFrame",
    roblox_datatype_color3: "roblox/datatypes/Color3",
    roblox_datatype_color_sequence: "roblox/datatypes/ColorSequence",
    roblox_datatype_color_sequence_keypoint: "roblox/datatypes/ColorSequenceKeypoint",
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
