use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::Axes as DomAxes;

use crate::{lune::util::TableBuilder, roblox::exports::LuaExportsTable};

use super::{super::*, EnumItem};

/**
    An implementation of the [Axes](https://create.roblox.com/docs/reference/engine/datatypes/Axes) Roblox datatype.

    This implements all documented properties, methods & constructors of the Axes class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axes {
    pub(crate) x: bool,
    pub(crate) y: bool,
    pub(crate) z: bool,
}

impl LuaExportsTable<'_> for Axes {
    const EXPORT_NAME: &'static str = "Axes";

    fn create_exports_table(lua: &Lua) -> LuaResult<LuaTable> {
        let axes_new = |_, args: LuaMultiValue| {
            let mut x = false;
            let mut y = false;
            let mut z = false;

            let mut check = |e: &EnumItem| {
                if e.parent.desc.name == "Axis" {
                    match &e.name {
                        name if name == "X" => x = true,
                        name if name == "Y" => y = true,
                        name if name == "Z" => z = true,
                        _ => {}
                    }
                } else if e.parent.desc.name == "NormalId" {
                    match &e.name {
                        name if name == "Left" || name == "Right" => x = true,
                        name if name == "Top" || name == "Bottom" => y = true,
                        name if name == "Front" || name == "Back" => z = true,
                        _ => {}
                    }
                }
            };

            for (index, arg) in args.into_iter().enumerate() {
                if let LuaValue::UserData(u) = arg {
                    if let Ok(e) = u.borrow::<EnumItem>() {
                        check(&e);
                    } else {
                        return Err(LuaError::RuntimeError(format!(
                            "Expected argument #{} to be an EnumItem, got userdata",
                            index
                        )));
                    }
                } else {
                    return Err(LuaError::RuntimeError(format!(
                        "Expected argument #{} to be an EnumItem, got {}",
                        index,
                        arg.type_name()
                    )));
                }
            }

            Ok(Axes { x, y, z })
        };

        TableBuilder::new(lua)?
            .with_function("new", axes_new)?
            .build_readonly()
    }
}

impl LuaUserData for Axes {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("X", |_, this| Ok(this.x));
        fields.add_field_method_get("Y", |_, this| Ok(this.y));
        fields.add_field_method_get("Z", |_, this| Ok(this.z));
        fields.add_field_method_get("Left", |_, this| Ok(this.x));
        fields.add_field_method_get("Right", |_, this| Ok(this.x));
        fields.add_field_method_get("Top", |_, this| Ok(this.y));
        fields.add_field_method_get("Bottom", |_, this| Ok(this.y));
        fields.add_field_method_get("Front", |_, this| Ok(this.z));
        fields.add_field_method_get("Back", |_, this| Ok(this.z));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for Axes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let write = make_list_writer();
        write(f, self.x, "X")?;
        write(f, self.y, "Y")?;
        write(f, self.z, "Z")?;
        Ok(())
    }
}

impl From<DomAxes> for Axes {
    fn from(v: DomAxes) -> Self {
        let bits = v.bits();
        Self {
            x: (bits & 1) == 1,
            y: ((bits >> 1) & 1) == 1,
            z: ((bits >> 2) & 1) == 1,
        }
    }
}

impl From<Axes> for DomAxes {
    fn from(v: Axes) -> Self {
        let mut bits = 0;
        bits += v.x as u8;
        bits += (v.y as u8) << 1;
        bits += (v.z as u8) << 2;
        DomAxes::from_bits(bits).expect("Invalid bits")
    }
}
