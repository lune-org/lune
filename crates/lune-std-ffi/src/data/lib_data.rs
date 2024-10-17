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

pub struct LibData(Library);

impl LibData {
    pub fn new(libname: String) -> LuaResult<Self> {
        match Library::open(libname) {
            Ok(t) => Ok(Self(t)),
            Err(err) => Err(LuaError::external(format!("{err}"))),
        }
    }

    pub fn get_sym<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
        name: String,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let lib = this.borrow::<LibData>()?;
        let sym = unsafe {
            lib.0
                .symbol::<*const ()>(name.as_str())
                .map_err(|err| LuaError::external(format!("{err}")))?
        };

        let ffi_ref =
            lua.create_userdata(RefData::new(sym.cast_mut(), LIB_REF_FLAGS, UNSIZED_BOUNDS))?;

        association::set(lua, SYM_INNER, &ffi_ref, &this)?;

        Ok(ffi_ref)
    }
}

impl LuaUserData for LibData {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("find", |lua, (this, name): (LuaAnyUserData, String)| {
            LibData::get_sym(lua, this, name)
        });
    }
}
