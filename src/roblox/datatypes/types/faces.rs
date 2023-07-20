use core::fmt;

use mlua::prelude::*;
use rbx_dom_weak::types::Faces as DomFaces;

use super::{super::*, EnumItem};

/**
    An implementation of the [Faces](https://create.roblox.com/docs/reference/engine/datatypes/Faces) Roblox datatype.

    This implements all documented properties, methods & constructors of the Faces class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Faces {
    pub(crate) right: bool,
    pub(crate) top: bool,
    pub(crate) back: bool,
    pub(crate) left: bool,
    pub(crate) bottom: bool,
    pub(crate) front: bool,
}

impl Faces {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        datatype_table.set(
            "new",
            lua.create_function(|_, args: LuaMultiValue| {
                let mut right = false;
                let mut top = false;
                let mut back = false;
                let mut left = false;
                let mut bottom = false;
                let mut front = false;
                let mut check = |e: &EnumItem| {
                    if e.parent.desc.name == "NormalId" {
                        match &e.name {
                            name if name == "Right" => right = true,
                            name if name == "Top" => top = true,
                            name if name == "Back" => back = true,
                            name if name == "Left" => left = true,
                            name if name == "Bottom" => bottom = true,
                            name if name == "Front" => front = true,
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
                Ok(Faces {
                    right,
                    top,
                    back,
                    left,
                    bottom,
                    front,
                })
            })?,
        )?;
        Ok(())
    }
}

impl LuaUserData for Faces {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Right", |_, this| Ok(this.right));
        fields.add_field_method_get("Top", |_, this| Ok(this.top));
        fields.add_field_method_get("Back", |_, this| Ok(this.back));
        fields.add_field_method_get("Left", |_, this| Ok(this.left));
        fields.add_field_method_get("Bottom", |_, this| Ok(this.bottom));
        fields.add_field_method_get("Front", |_, this| Ok(this.front));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for Faces {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let write = make_list_writer();
        write(f, self.right, "Right")?;
        write(f, self.top, "Top")?;
        write(f, self.back, "Back")?;
        write(f, self.left, "Left")?;
        write(f, self.bottom, "Bottom")?;
        write(f, self.front, "Front")?;
        Ok(())
    }
}

impl From<DomFaces> for Faces {
    fn from(v: DomFaces) -> Self {
        let bits = v.bits();
        Self {
            right: (bits & 1) == 1,
            top: ((bits >> 1) & 1) == 1,
            back: ((bits >> 2) & 1) == 1,
            left: ((bits >> 3) & 1) == 1,
            bottom: ((bits >> 4) & 1) == 1,
            front: ((bits >> 5) & 1) == 1,
        }
    }
}

impl From<Faces> for DomFaces {
    fn from(v: Faces) -> Self {
        let mut bits = 0;
        bits += v.right as u8;
        bits += (v.top as u8) << 1;
        bits += (v.back as u8) << 2;
        bits += (v.left as u8) << 3;
        bits += (v.bottom as u8) << 4;
        bits += (v.front as u8) << 5;
        DomFaces::from_bits(bits).expect("Invalid bits")
    }
}
