use core::fmt;
use std::ops;

use glam::Vec2;
use mlua::prelude::*;
use rbx_dom_weak::types::Rect as DomRect;

use super::{super::*, Vector2};

/**
    An implementation of the [Rect](https://create.roblox.com/docs/reference/engine/datatypes/Rect)
    Roblox datatype, backed by [`glam::Vec2`].

    This implements all documented properties, methods & constructors of the Rect class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub(crate) min: Vec2,
    pub(crate) max: Vec2,
}

impl Rect {
    fn new(lhs: Vec2, rhs: Vec2) -> Self {
        Self {
            min: lhs.min(rhs),
            max: lhs.max(rhs),
        }
    }
}

impl Rect {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        type ArgsVector2s = (Option<Vector2>, Option<Vector2>);
        type ArgsNums = (Option<f32>, Option<f32>, Option<f32>, Option<f32>);
        datatype_table.set(
            "new",
            lua.create_function(|lua, args: LuaMultiValue| {
                if let Ok((min, max)) = ArgsVector2s::from_lua_multi(args.clone(), lua) {
                    Ok(Rect::new(
                        min.unwrap_or_default().0,
                        max.unwrap_or_default().0,
                    ))
                } else if let Ok((x0, y0, x1, y1)) = ArgsNums::from_lua_multi(args, lua) {
                    let min = Vec2::new(x0.unwrap_or_default(), y0.unwrap_or_default());
                    let max = Vec2::new(x1.unwrap_or_default(), y1.unwrap_or_default());
                    Ok(Rect::new(min, max))
                } else {
                    // FUTURE: Better error message here using given arg types
                    Err(LuaError::RuntimeError(
                        "Invalid arguments to constructor".to_string(),
                    ))
                }
            })?,
        )
    }
}

impl LuaUserData for Rect {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Min", |_, this| Ok(Vector2(this.min)));
        fields.add_field_method_get("Max", |_, this| Ok(Vector2(this.max)));
        fields.add_field_method_get("Width", |_, this| Ok(this.max.x - this.min.x));
        fields.add_field_method_get("Height", |_, this| Ok(this.max.y - this.min.y));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Unm, userdata_impl_unm);
        methods.add_meta_method(LuaMetaMethod::Add, userdata_impl_add);
        methods.add_meta_method(LuaMetaMethod::Sub, userdata_impl_sub);
    }
}

impl fmt::Display for Rect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.min, self.max)
    }
}

impl ops::Neg for Rect {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Rect::new(-self.min, -self.max)
    }
}

impl ops::Add for Rect {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Rect::new(self.min + rhs.min, self.max + rhs.max)
    }
}

impl ops::Sub for Rect {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Rect::new(self.min - rhs.min, self.max - rhs.max)
    }
}

impl From<DomRect> for Rect {
    fn from(v: DomRect) -> Self {
        Rect {
            min: Vec2::new(v.min.x, v.min.y),
            max: Vec2::new(v.max.x, v.max.y),
        }
    }
}

impl From<Rect> for DomRect {
    fn from(v: Rect) -> Self {
        DomRect {
            min: Vector2(v.min).into(),
            max: Vector2(v.max).into(),
        }
    }
}
