mod axes;
mod brick_color;
mod cframe;
mod color3;
mod color_sequence;
mod color_sequence_keypoint;
mod r#enum;
mod r#enum_item;
mod r#enums;
mod faces;
mod number_range;
mod number_sequence;
mod number_sequence_keypoint;
mod physical_properties;
mod ray;
mod rect;
mod region3;
mod region3int16;
mod udim;
mod udim2;
mod vector2;
mod vector2int16;
mod vector3;
mod vector3int16;

pub use axes::Axes;
pub use brick_color::BrickColor;
pub use cframe::CFrame;
pub use color3::Color3;
pub use color_sequence::ColorSequence;
pub use color_sequence_keypoint::ColorSequenceKeypoint;
pub use faces::Faces;
pub use number_range::NumberRange;
pub use number_sequence::NumberSequence;
pub use number_sequence_keypoint::NumberSequenceKeypoint;
pub use physical_properties::PhysicalProperties;
pub use r#enum::Enum;
pub use r#enum_item::EnumItem;
pub use r#enums::Enums;
pub use ray::Ray;
pub use rect::Rect;
pub use region3::Region3;
pub use region3int16::Region3int16;
pub use udim::UDim;
pub use udim2::UDim2;
pub use vector2::Vector2;
pub use vector2int16::Vector2int16;
pub use vector3::Vector3;
pub use vector3int16::Vector3int16;

#[cfg(test)]
mod tests {
    use std::{env::set_current_dir, fs::read_to_string, path::PathBuf};

    use anyhow::{Context, Result};
    use mlua::prelude::*;

    use crate::make_all_datatypes;

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
        axes:                     "datatypes/Axes",
        brick_color:              "datatypes/BrickColor",
        cframe:                   "datatypes/CFrame",
        color3:                   "datatypes/Color3",
        color_sequence:           "datatypes/ColorSequence",
        color_sequence_keypoint:  "datatypes/ColorSequenceKeypoint",
        r#enum:                   "datatypes/Enum",
        faces:                    "datatypes/Faces",
        number_range:             "datatypes/NumberRange",
        number_sequence:          "datatypes/NumberSequence",
        number_sequence_keypoint: "datatypes/NumberSequenceKeypoint",
        physical_properties:      "datatypes/PhysicalProperties",
        ray:                      "datatypes/Ray",
        rect:                     "datatypes/Rect",
        udim:                     "datatypes/UDim",
        udim2:                    "datatypes/UDim2",
        region3:                  "datatypes/Region3",
        region3int16:             "datatypes/Region3int16",
        vector2:                  "datatypes/Vector2",
        vector2int16:             "datatypes/Vector2int16",
        vector3:                  "datatypes/Vector3",
        vector3int16:             "datatypes/Vector3int16",
    }
}
