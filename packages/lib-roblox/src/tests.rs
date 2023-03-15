use std::{env::set_current_dir, fs::read_to_string, path::PathBuf};

use anyhow::{Context, Result};
use mlua::prelude::*;

use super::make_all_datatypes;

macro_rules! create_tests {
    ($($test_name:ident: $file_path:expr,)*) => { $(
        #[test]
        fn $test_name() -> Result<()> {
			// NOTE: This path is relative to the lib
			// package, not the cwd or workspace root,
			// so we need to cd to the repo root first
			let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
			let root_dir = crate_dir.join("../../").canonicalize()?;
			set_current_dir(root_dir)?;
			// Create all datatypes as globals
			let lua = Lua::new();
			let env = lua.globals();
			for (name, tab) in make_all_datatypes(&lua)? {
				env.set(name, tab)?;
			}
			// The rest of the test logic can continue as normal
			let full_name = format!("tests/roblox/{}.luau", $file_path);
			let script = read_to_string(full_name)
				.with_context(|| format!(
					"Failed to read test file '{}'",
					$file_path
				))?;
			lua.load(&script)
				.set_name($file_path)?
				.set_environment(env)?
				.exec()?;
			Ok(())
        }
    )* }
}

create_tests! {
    datatypes_brick_color:  "datatypes/BrickColor",
    datatypes_color3:       "datatypes/Color3",
    datatypes_udim:         "datatypes/UDim",
    datatypes_udim2:        "datatypes/UDim2",
    datatypes_vector2:      "datatypes/Vector2",
    datatypes_vector2int16: "datatypes/Vector2int16",
    datatypes_vector3:      "datatypes/Vector3",
    datatypes_vector3int16: "datatypes/Vector3int16",
}
