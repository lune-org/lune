use std::fmt;

use lune_utils::TableBuilder;
use mlua::BString;
use mlua::prelude::*;

use super::super::*;
use crate::exports::LuaExportsTable;

use rbx_dom_weak::types::UniqueId as DomUniqueId;

/**
    An implementation of the `UniqueId` Roblox datatype.

    This type is not exposed to users in engine by Roblox itself,
    but is used as an identifier for Instances, and is occasionally
    useful when manipulating place and model files in Lune.
*/
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UniqueId {
    id: u128,
}

impl LuaExportsTable for UniqueId {
    const EXPORT_NAME: &'static str = "UniqueId";

    fn create_exports_table(lua: Lua) -> LuaResult<LuaTable> {
        let from_string = |_: &Lua, input: BString| {
            if input.len() == 16 {
                let mut bytes = [0; 16];
                bytes.copy_from_slice(&input);
                Ok(UniqueId {
                    id: u128::from_be_bytes(bytes),
                })
            } else {
                Err(LuaError::RuntimeError(format!(
                    "UniqueId.fromString expects 16 bytes but {} bytes were provided",
                    input.len()
                )))
            }
        };

        let new = |_: &Lua, ()| match DomUniqueId::now() {
            Ok(uid) => Ok(UniqueId::from(uid)),
            Err(err) => Err(LuaError::RuntimeError(format!(
                "UniqueId.new encountered an error: {err}"
            ))),
        };

        TableBuilder::new(lua)?
            .with_function("fromString", from_string)?
            .with_function("new", new)?
            .with_value("null", UniqueId { id: 0 })?
            .build_readonly()
    }
}

impl LuaUserData for UniqueId {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
    }
}

impl fmt::Display for UniqueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.id.to_be_bytes() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl From<DomUniqueId> for UniqueId {
    fn from(value: DomUniqueId) -> Self {
        let mut bytes = [0; 16];
        bytes[0..8].copy_from_slice(&value.random().to_be_bytes());
        bytes[8..12].copy_from_slice(&value.time().to_be_bytes());
        bytes[12..16].copy_from_slice(&value.index().to_be_bytes());
        Self {
            id: u128::from_be_bytes(bytes),
        }
    }
}

impl From<UniqueId> for DomUniqueId {
    fn from(value: UniqueId) -> Self {
        let bytes = value.id.to_be_bytes();
        DomUniqueId::new(
            u32::from_be_bytes(bytes[12..16].try_into().unwrap()),
            u32::from_be_bytes(bytes[8..12].try_into().unwrap()),
            i64::from_be_bytes(bytes[0..8].try_into().unwrap()),
        )
    }
}
