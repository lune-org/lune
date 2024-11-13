use dlopen2::raw::Library;
use mlua::prelude::*;

use super::{
    association_names::SYM_INNER,
    ref_data::{RefData, RefFlag, UNSIZED_BOUNDS},
};
use crate::ffi::association;

const LIB_REF_FLAGS: u8 = RefFlag::Offsetable.value()
    | RefFlag::Readable.value()
    | RefFlag::Dereferenceable.value()
    | RefFlag::Function.value();

// Runtime dynamic loaded libraries
pub struct LibData {
    name: String,
    lib: Library,
}

impl LibData {
    // Open library then return library handle
    pub fn new(libname: String) -> LuaResult<Self> {
        match Library::open(&libname) {
            Ok(t) => Ok(Self {
                lib: t,
                name: libname.clone(),
            }),
            Err(err) => Err(err.into_lua_err()),
        }
    }

    // Get named symbol from library
    pub fn find_symbol<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
        name: String,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let lib = this.borrow::<LibData>()?;
        let sym = unsafe {
            lib.lib
                .symbol::<*const ()>(name.as_str())
                .map_err(LuaError::external)?
        };
        let ffi_ref =
            lua.create_userdata(RefData::new(sym.cast_mut(), LIB_REF_FLAGS, UNSIZED_BOUNDS))?;

        // Library handle should live longer than retrieved symbol
        association::set(lua, SYM_INNER, &ffi_ref, &this)?;

        Ok(ffi_ref)
    }

    pub fn stringify(&self) -> String {
        self.name.clone()
    }
}

impl LuaUserData for LibData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("find", |lua, (this, name): (LuaAnyUserData, String)| {
            LibData::find_symbol(lua, this, name)
        });
        methods.add_meta_method(LuaMetaMethod::ToString, |_lua, this, ()| {
            Ok(this.stringify())
        });
    }
}
