use core::fmt;
use std::ops;

use mlua::prelude::*;
use rbx_dom_weak::types::UDim as RbxUDim;

use super::super::*;

/**
    An implementation of the [UDim](https://create.roblox.com/docs/reference/engine/datatypes/UDim) Roblox datatype.

    This implements all documented properties, methods & constructors of the UDim class as of March 2023.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UDim {
    pub(crate) scale: f32,
    pub(crate) offset: i32,
}

impl UDim {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        datatype_table.set(
            "new",
            lua.create_function(|_, (scale, offset): (Option<f32>, Option<i32>)| {
                Ok(UDim {
                    scale: scale.unwrap_or_default(),
                    offset: offset.unwrap_or_default(),
                })
            })?,
        )
    }
}

impl Default for UDim {
    fn default() -> Self {
        Self {
            scale: 0f32,
            offset: 0,
        }
    }
}

impl fmt::Display for UDim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.scale, self.offset)
    }
}

impl ops::Neg for UDim {
    type Output = Self;
    fn neg(self) -> Self::Output {
        UDim {
            scale: -self.scale,
            offset: -self.offset,
        }
    }
}

impl ops::Add for UDim {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        UDim {
            scale: self.scale + rhs.scale,
            offset: self.offset + rhs.offset,
        }
    }
}

impl ops::Sub for UDim {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        UDim {
            scale: self.scale - rhs.scale,
            offset: self.offset - rhs.offset,
        }
    }
}

impl LuaUserData for UDim {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("Scale", |_, this| Ok(this.scale));
        fields.add_field_method_get("Offset", |_, this| Ok(this.offset));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Unm, |_, this, ()| Ok(-*this));
        methods.add_meta_method(LuaMetaMethod::Add, |_, this, rhs: UDim| Ok(*this + rhs));
        methods.add_meta_method(LuaMetaMethod::Sub, |_, this, rhs: UDim| Ok(*this - rhs));
    }
}

impl From<&RbxUDim> for UDim {
    fn from(v: &RbxUDim) -> Self {
        UDim {
            scale: v.scale,
            offset: v.offset,
        }
    }
}

impl From<&UDim> for RbxUDim {
    fn from(v: &UDim) -> Self {
        RbxUDim {
            scale: v.scale,
            offset: v.offset,
        }
    }
}

impl FromRbxVariant for UDim {
    fn from_rbx_variant(variant: &RbxVariant) -> DatatypeConversionResult<Self> {
        if let RbxVariant::UDim(u) = variant {
            Ok(u.into())
        } else {
            Err(DatatypeConversionError::FromRbxVariant {
                from: variant.variant_name(),
                to: "UDim",
                detail: None,
            })
        }
    }
}

impl ToRbxVariant for UDim {
    fn to_rbx_variant(
        &self,
        desired_type: Option<RbxVariantType>,
    ) -> DatatypeConversionResult<RbxVariant> {
        if matches!(desired_type, None | Some(RbxVariantType::UDim)) {
            Ok(RbxVariant::UDim(self.into()))
        } else {
            Err(DatatypeConversionError::ToRbxVariant {
                to: desired_type.map(|d| d.variant_name()).unwrap_or("?"),
                from: "UDim",
                detail: None,
            })
        }
    }
}
