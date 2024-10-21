use libffi::middle::Type;
use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::{ctype_helper, void_info::CVoidInfo, CArrInfo, CFnInfo, CPtrInfo, CStructInfo};
use crate::{
    data::{BoxData, GetFfiData},
    ffi::{FfiConvert, FfiSize},
};

pub mod method_provider {
    use super::*;

    pub fn provide_to_string<'lua, Target, M>(methods: &mut M)
    where
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_meta_function(LuaMetaMethod::ToString, |lua, this: LuaAnyUserData| {
            stringify(lua, &this)
        });
    }

    pub fn provide_ptr<'lua, Target, M>(methods: &mut M)
    where
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_function("ptr", |lua, this: LuaAnyUserData| {
            CPtrInfo::from_userdata(lua, &this)
        });
    }

    pub fn provide_arr<'lua, Target, M>(methods: &mut M)
    where
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_function("arr", |lua, (this, length): (LuaAnyUserData, usize)| {
            CArrInfo::from_userdata(lua, &this, length)
        });
    }

    pub fn provide_read_data<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiSize + FfiConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "readData",
            |lua, this, (target, offset): (LuaAnyUserData, Option<isize>)| {
                let offset = offset.unwrap_or(0);

                let data_handle = &target.get_ffi_data()?;
                if !data_handle.check_inner_boundary(offset, this.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !data_handle.is_readable() {
                    return Err(LuaError::external("Unreadable data handle"));
                }

                unsafe { this.value_from_data(lua, offset, data_handle) }
            },
        );
    }

    pub fn provide_write_data<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiSize + FfiConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "writeData",
            |lua, this, (target, value, offset): (LuaAnyUserData, LuaValue, Option<isize>)| {
                let offset = offset.unwrap_or(0);

                let data_handle = &target.get_ffi_data()?;
                // use or functions
                if !data_handle.check_inner_boundary(offset, this.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !data_handle.is_writable() {
                    return Err(LuaError::external("Unwritable data handle"));
                }

                unsafe { this.value_into_data(lua, offset, data_handle, value) }
            },
        );
    }

    pub fn provide_copy_data<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiSize + FfiConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "copyData",
            |lua,
             this,
             (dst, src, dst_offset, src_offset): (
                LuaAnyUserData,
                LuaAnyUserData,
                Option<isize>,
                Option<isize>,
            )| {
                let dst_offset = dst_offset.unwrap_or(0);
                let src_offset = src_offset.unwrap_or(0);

                let dst = &dst.get_ffi_data()?;
                // use or functions
                if !dst.check_inner_boundary(dst_offset, this.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !dst.is_writable() {
                    return Err(LuaError::external("Unwritable data handle"));
                }

                let src = &src.get_ffi_data()?;
                if !src.check_inner_boundary(dst_offset, this.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !src.is_readable() {
                    return Err(LuaError::external("Unreadable value data handle"));
                }

                unsafe { this.copy_data(lua, dst_offset, src_offset, dst, src) }
            },
        );
    }

    pub fn provide_stringify_data<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiSize + FfiConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "stringifyData",
            |lua, this, (target, offset): (LuaAnyUserData, Option<isize>)| unsafe {
                this.stringify_data(lua, offset.unwrap_or(0), &target.get_ffi_data()?)
            },
        );
    }

    pub fn provide_box<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiSize + FfiConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method("box", |lua, this, value: LuaValue| {
            let result = lua.create_userdata(BoxData::new(this.get_size()))?;
            unsafe { this.value_into_data(lua, 0, &result.get_ffi_data()?, value)? };
            Ok(result)
        });
    }

    // FIXME: Buffer support should be part of another PR
    // pub fn provide_write_buffer<'lua, Target, M>(methods: &mut M)
    // where
    //     Target: FfiSize + FfiConvert,
    //     M: LuaUserDataMethods<'lua, Target>,
    // {
    //     methods.add_method(
    //         "writeBuffer",
    //         |lua, this, (target, value, offset): (LuaValue, LuaValue, Option<isize>)| {
    //             if !target.is_buffer() {
    //                 return Err(LuaError::external(format!(
    //                     "Argument target must be a buffer, got {}",
    //                     target.type_name()
    //                 )));
    //             }

    //             target.to_pointer()
    //             target.as_userdata().unwrap().to_pointer()
    //             let offset = offset.unwrap_or(0);

    //             let data_handle = &target.get_ffi_data()?;
    //             // use or functions
    //             if !data_handle.check_boundary(offset, this.get_size()) {
    //                 return Err(LuaError::external("Out of bounds"));
    //             }
    //             if !data_handle.is_writable() {
    //                 return Err(LuaError::external("Unwritable data handle"));
    //             }

    //             unsafe { this.value_into_data(lua, offset, data_handle, value) }
    //         },
    //     );
    // }
}

pub fn get_userdata(value: LuaValue) -> LuaResult<LuaAnyUserData> {
    if let LuaValue::UserData(field_type) = value {
        Ok(field_type)
    } else {
        Err(LuaError::external(format!(
            "CStruct, CType, CFn, CVoid or CArr is required but got {}",
            pretty_format_value(&value, &ValueFormatConfig::new())
        )))
    }
}

// Get the NativeConvert handle from the type UserData
// this is intended to avoid lookup userdata and lua table every time. (eg: struct)
// userdata must live longer than the NativeConvert handle.
// However, c_struct is a strong reference to each field, so this is not a problem.
pub unsafe fn get_conv(userdata: &LuaAnyUserData) -> LuaResult<*const dyn FfiConvert> {
    if userdata.is::<CStructInfo>() {
        Ok(userdata.to_pointer().cast::<CStructInfo>() as *const dyn FfiConvert)
    } else if userdata.is::<CArrInfo>() {
        Ok(userdata.to_pointer().cast::<CArrInfo>() as *const dyn FfiConvert)
    } else if userdata.is::<CPtrInfo>() {
        Ok(userdata.to_pointer().cast::<CPtrInfo>() as *const dyn FfiConvert)
    } else {
        ctype_helper::get_conv(userdata)
    }
}

// Create vec<T> from table with (userdata)->T
pub fn create_list<T>(
    table: &LuaTable,
    callback: fn(&LuaAnyUserData) -> LuaResult<T>,
) -> LuaResult<Vec<T>> {
    let len: usize = table.raw_len();
    let mut list = Vec::<T>::with_capacity(len);

    for i in 0..len {
        let value: LuaValue = table.raw_get(i + 1)?;
        list.push(callback(&get_userdata(value)?)?);
    }

    Ok(list)
}

//Get
pub unsafe fn get_conv_list(table: &LuaTable) -> LuaResult<Vec<*const dyn FfiConvert>> {
    create_list(table, |userdata| get_conv(userdata))
}

// Get type size from ctype userdata
pub fn get_size(userdata: &LuaAnyUserData) -> LuaResult<usize> {
    if userdata.is::<CStructInfo>() {
        Ok(userdata.borrow::<CStructInfo>()?.get_size())
    } else if userdata.is::<CArrInfo>() {
        Ok(userdata.borrow::<CArrInfo>()?.get_size())
    } else if userdata.is::<CPtrInfo>() {
        Ok(userdata.borrow::<CPtrInfo>()?.get_size())
    } else if userdata.is::<CVoidInfo>() {
        Ok(userdata.borrow::<CVoidInfo>()?.get_size())
    } else if userdata.is::<CFnInfo>() {
        Ok(userdata.borrow::<CFnInfo>()?.get_size())
    } else {
        ctype_helper::get_size(userdata)
    }
}

// Get libffi_type from ctype userdata
pub fn get_middle_type(userdata: &LuaAnyUserData) -> LuaResult<Type> {
    if userdata.is::<CStructInfo>() {
        Ok(userdata.borrow::<CStructInfo>()?.get_middle_type())
    } else if let Some(middle_type) = ctype_helper::get_middle_type(userdata)? {
        Ok(middle_type)
    } else if userdata.is::<CArrInfo>() {
        Ok(userdata.borrow::<CArrInfo>()?.get_middle_type())
    } else if userdata.is::<CPtrInfo>() {
        Ok(CPtrInfo::get_middle_type())
    } else if userdata.is::<CVoidInfo>() {
        Ok(CVoidInfo::get_middle_type())
    } else if userdata.is::<CFnInfo>() {
        Ok(CFnInfo::get_middle_type())
    } else {
        Err(LuaError::external(format!(
            "CStruct, CType, CFn, CVoid or CArr is required but got {}",
            pretty_format_value(
                // Since the data is in the Lua location,
                // there is no problem with the clone.
                &LuaValue::UserData(userdata.to_owned()),
                &ValueFormatConfig::new()
            )
        )))
    }
}

// get Vec<libffi_type> from table(array) of c-type userdata
pub fn get_middle_type_list(table: &LuaTable) -> LuaResult<Vec<Type>> {
    create_list(table, get_middle_type)
}

pub fn has_void(table: &LuaTable) -> LuaResult<bool> {
    for i in 0..table.raw_len() {
        let value: LuaValue = table.raw_get(i + 1)?;
        if get_userdata(value)?.is::<CVoidInfo>() {
            return Ok(false);
        }
    }
    Ok(false)
}

// stringify any c-type userdata (for recursive)
pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
    if userdata.is::<CStructInfo>() {
        CStructInfo::stringify(lua, userdata)
    } else if userdata.is::<CArrInfo>() {
        CArrInfo::stringify(lua, userdata)
    } else if userdata.is::<CPtrInfo>() {
        CPtrInfo::stringify(lua, userdata)
    } else if userdata.is::<CFnInfo>() {
        CFnInfo::stringify(lua, userdata)
    } else if let Some(name) = ctype_helper::get_name(userdata)? {
        Ok(String::from(name))
    } else {
        Ok(String::from("unknown"))
    }
}

// get name tag for any c-type userdata
pub fn get_tag_name(userdata: &LuaAnyUserData) -> LuaResult<String> {
    Ok(if userdata.is::<CStructInfo>() {
        String::from("CStruct")
    } else if userdata.is::<CArrInfo>() {
        String::from("CArr")
    } else if userdata.is::<CPtrInfo>() {
        String::from("CPtr")
    } else if userdata.is::<CFnInfo>() {
        String::from("CFn")
    } else if userdata.is::<CVoidInfo>() {
        String::from("CVoid")
    } else if ctype_helper::is_ctype(userdata) {
        String::from("CType")
    } else {
        String::from("Unknown")
    })
}

// emulate 'print' for ctype userdata, but ctype is simplified
pub fn pretty_format(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
    if ctype_helper::is_ctype(userdata) {
        stringify(lua, userdata)
    } else {
        Ok(format!(
            "<{}({})>",
            get_tag_name(userdata)?,
            stringify(lua, userdata)?
        ))
    }
}
