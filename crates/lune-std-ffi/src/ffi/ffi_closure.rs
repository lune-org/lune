use core::ffi::c_void;
use std::ptr;

use libffi::{
    low::{closure_alloc, closure_free, ffi_cif, CodePtr},
    raw::{ffi_closure, ffi_prep_closure_loc},
};
use mlua::prelude::*;

use super::{
    ffi_ref::{FfiRefBounds, FfiRefFlag},
    FfiRef, FFI_STATUS_NAMES,
};

pub struct FfiClosure<'a> {
    closure: *mut ffi_closure,
    code: CodePtr,
    userdata: CallbackUserdata<'a>,
}

impl<'a> Drop for FfiClosure<'a> {
    fn drop(&mut self) {
        unsafe {
            closure_free(self.closure);
        }
    }
}

#[allow(unused)]
pub struct CallbackUserdata<'a> {
    pub func: LuaFunction<'a>,
    pub lua: &'a Lua,
    pub arg_ref_flags: Vec<u8>,
    pub arg_ref_size: Vec<usize>,
    pub result_size: usize,
}

const RESULT_REF_FLAGS: u8 = FfiRefFlag::Leaked.value() | FfiRefFlag::Writable.value();

unsafe extern "C" fn callback(
    cif: *mut ffi_cif,
    result_pointer: *mut c_void,
    arg_pointers: *mut *mut c_void,
    userdata: *mut c_void,
) {
    let userdata = userdata.cast::<CallbackUserdata>();
    let len = (*cif).nargs as usize;
    let mut args = Vec::<LuaValue>::with_capacity(len + 1);

    // Push result pointer (ref)
    args.push(LuaValue::UserData(
        (*userdata)
            .lua
            .create_userdata(FfiRef::new(
                result_pointer.cast::<()>(),
                RESULT_REF_FLAGS,
                FfiRefBounds::new(0, (*userdata).result_size),
            ))
            .unwrap(),
    ));

    // Push arg pointer (ref)
    for i in 0..len {
        args.push(LuaValue::UserData(
            (*userdata)
                .lua
                .create_userdata(FfiRef::new(
                    (*arg_pointers.add(i)).cast::<()>(),
                    (*userdata).arg_ref_flags.get(i).unwrap().to_owned(),
                    FfiRefBounds::new(0, (*userdata).arg_ref_size.get(i).unwrap().to_owned()),
                ))
                .unwrap(),
        ));
    }

    (*userdata).func.call::<_, ()>(args).unwrap();
}

impl<'a> FfiClosure<'a> {
    pub unsafe fn new(
        cif: *mut ffi_cif,
        userdata: CallbackUserdata<'a>,
    ) -> LuaResult<FfiClosure<'a>> {
        let (closure, code) = closure_alloc();
        let prep_result = ffi_prep_closure_loc(
            closure,
            cif,
            Some(callback),
            ptr::from_ref(&userdata).cast::<c_void>().cast_mut(),
            code.as_mut_ptr(),
        );

        if prep_result != 0 {
            Err(LuaError::external(format!(
                "ffi_get_struct_offsets failed. expected result {}, got {}",
                FFI_STATUS_NAMES[0], FFI_STATUS_NAMES[prep_result as usize]
            )))
        } else {
            Ok(FfiClosure {
                closure,
                code,
                userdata,
            })
        }
    }
}
