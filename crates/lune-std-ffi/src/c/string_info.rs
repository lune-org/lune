// TODO:

use mlua::prelude::*;

pub struct CStringInfo();

impl CStringInfo {
    pub fn new() -> Self {
        Self()
    }
}

impl LuaUserData for CStringInfo {}
