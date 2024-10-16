use dlopen2::raw::Library;
use mlua::prelude::*;

use super::{
    association_names::SYM_INNER,
    ffi_association::set_association,
    ffi_ref::{FfiRef, FfiRefFlag, UNSIZED_BOUNDS},
};

const LIB_REF_FLAGS: u8 = FfiRefFlag::Offsetable.value()
    | FfiRefFlag::Readable.value()
    | FfiRefFlag::Dereferenceable.value()
    | FfiRefFlag::Function.value();

pub struct FfiLib(Library);

impl FfiLib {
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
        let lib = this.borrow::<FfiLib>()?;
        let sym = unsafe {
            lib.0
                .symbol::<*const ()>(name.as_str())
                .map_err(|err| LuaError::external(format!("{err}")))?
        };

        let ffi_ref =
            lua.create_userdata(FfiRef::new(sym.cast_mut(), LIB_REF_FLAGS, UNSIZED_BOUNDS))?;

        set_association(lua, SYM_INNER, &ffi_ref, &this)?;

        Ok(ffi_ref)
    }
}

impl LuaUserData for FfiLib {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("find", |lua, (this, name): (LuaAnyUserData, String)| {
            FfiLib::get_sym(lua, this, name)
        });
    }
}
