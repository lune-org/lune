use libffi::middle::Type;
use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::{ctype_helper, CArrInfo, CFnInfo, CPtrInfo, CStructInfo};
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

    pub fn provide_ptr_info<'lua, Target, M>(methods: &mut M)
    where
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_function("pointerInfo", |lua, this: LuaAnyUserData| {
            CPtrInfo::from_userdata(lua, &this)
        });
    }

    pub fn provide_arr_info<'lua, Target, M>(methods: &mut M)
    where
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_function("ArrInfo", |lua, (this, length): (LuaAnyUserData, usize)| {
            CArrInfo::from_userdata(lua, &this, length)
        });
    }

    pub fn provide_from_data<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiSize + FfiConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "fromData",
            |lua, this, (userdata, offset): (LuaAnyUserData, Option<isize>)| {
                let offset = offset.unwrap_or(0);

                let data_handle = &userdata.get_data_handle()?;
                if !data_handle.check_boundary(offset, this.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !data_handle.is_readable() {
                    return Err(LuaError::external("Unreadable data handle"));
                }

                unsafe { this.value_from_data(lua, offset, data_handle) }
            },
        );
    }

    pub fn provide_into_data<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiSize + FfiConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "intoData",
            |lua, this, (userdata, value, offset): (LuaAnyUserData, LuaValue, Option<isize>)| {
                let offset = offset.unwrap_or(0);

                let data_handle = &userdata.get_data_handle()?;
                // use or functions
                if !data_handle.check_boundary(offset, this.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !data_handle.is_writable() {
                    return Err(LuaError::external("Unwritable data handle"));
                }

                unsafe { this.value_into_data(lua, offset, data_handle, value) }
            },
        );
    }

    pub fn provide_box<'lua, Target, M>(methods: &mut M)
    where
        Target: FfiSize + FfiConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method("box", |lua, this, table: LuaValue| {
            let result = lua.create_userdata(BoxData::new(this.get_size()))?;
            unsafe { this.value_into_data(lua, 0, &result.get_data_handle()?, table)? };
            Ok(result)
        });
    }
}

pub fn get_userdata(value: LuaValue) -> LuaResult<LuaAnyUserData> {
    if let LuaValue::UserData(field_type) = value {
        Ok(field_type)
    } else {
        Err(LuaError::external(format!(
            "Unexpected field. CStruct, CType or CArr is required for element but got {}",
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
        // TODO: struct and more
    }
}

pub unsafe fn get_conv_list(table: &LuaTable) -> LuaResult<Vec<*const dyn FfiConvert>> {
    let len: usize = table.raw_len();
    let mut conv_list = Vec::<*const dyn FfiConvert>::with_capacity(len);

    for i in 0..len {
        let value: LuaValue = table.raw_get(i + 1)?;
        conv_list.push(get_conv(&get_userdata(value)?)?);
    }

    Ok(conv_list)
}

pub fn get_size(this: &LuaAnyUserData) -> LuaResult<usize> {
    if this.is::<CStructInfo>() {
        Ok(this.borrow::<CStructInfo>()?.get_size())
    } else if this.is::<CArrInfo>() {
        Ok(this.borrow::<CArrInfo>()?.get_size())
    } else if this.is::<CPtrInfo>() {
        Ok(this.borrow::<CPtrInfo>()?.get_size())
    } else {
        ctype_helper::get_size(this)
    }
}

// get libffi_type from any c-type userdata
pub fn get_middle_type(userdata: &LuaAnyUserData) -> LuaResult<Type> {
    if userdata.is::<CStructInfo>() {
        Ok(userdata.borrow::<CStructInfo>()?.get_type())
    } else if let Some(middle_type) = ctype_helper::get_middle_type(userdata)? {
        Ok(middle_type)
    } else if userdata.is::<CArrInfo>() {
        Ok(userdata.borrow::<CArrInfo>()?.get_type())
    } else if userdata.is::<CPtrInfo>() {
        Ok(CPtrInfo::get_type())
    } else {
        Err(LuaError::external(format!(
            "Unexpected field. CStruct, CType, CString or CArr is required for element but got {}",
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
    let len: usize = table.raw_len();
    let mut fields = Vec::with_capacity(len);

    for i in 0..len {
        // Test required
        let value = table.raw_get(i + 1)?;
        if let LuaValue::UserData(field_type) = value {
            fields.push(get_middle_type(&field_type)?);
        } else {
            return Err(LuaError::external(format!(
                "Unexpected field. CStruct, CType or CArr is required for element but got {}",
                value.type_name()
            )));
        }
    }

    Ok(fields)
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
        String::from("CFunc")
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
