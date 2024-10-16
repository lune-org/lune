use libffi::middle::Type;
use lune_utils::fmt::{pretty_format_value, ValueFormatConfig};
use mlua::prelude::*;

use super::{c_type_helper, CArr, CFunc, CPtr, CStruct};
use crate::ffi::{FfiBox, GetNativeData, NativeConvert, NativeSize};

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
            CPtr::from_userdata(lua, &this)
        });
    }

    pub fn provide_arr<'lua, Target, M>(methods: &mut M)
    where
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_function("arr", |lua, (this, length): (LuaAnyUserData, usize)| {
            CArr::from_userdata(lua, &this, length)
        });
    }

    pub fn provide_from<'lua, Target, M>(methods: &mut M)
    where
        Target: NativeSize + NativeConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "from",
            |lua, this, (userdata, offset): (LuaAnyUserData, Option<isize>)| {
                let offset = offset.unwrap_or(0);

                let data_handle = &userdata.get_data_handle()?;
                if !data_handle.check_boundary(offset, this.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }

                unsafe { this.luavalue_from(lua, offset, data_handle) }
            },
        );
    }

    pub fn provide_into<'lua, Target, M>(methods: &mut M)
    where
        Target: NativeSize + NativeConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method(
            "into",
            |lua, this, (userdata, value, offset): (LuaAnyUserData, LuaValue, Option<isize>)| {
                let offset = offset.unwrap_or(0);

                let data_handle = &userdata.get_data_handle()?;
                if !data_handle.check_boundary(offset, this.get_size()) {
                    return Err(LuaError::external("Out of bounds"));
                }
                if !data_handle.is_writable() {
                    return Err(LuaError::external("Unwritable data handle"));
                }

                unsafe { this.luavalue_into(lua, offset, data_handle, value) }
            },
        );
    }

    pub fn provide_box<'lua, Target, M>(methods: &mut M)
    where
        Target: NativeSize + NativeConvert,
        M: LuaUserDataMethods<'lua, Target>,
    {
        methods.add_method("box", |lua, this, table: LuaValue| {
            let result = lua.create_userdata(FfiBox::new(this.get_size()))?;
            unsafe { this.luavalue_into(lua, 0, &result.get_data_handle()?, table)? };
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
pub unsafe fn get_conv(userdata: &LuaAnyUserData) -> LuaResult<*const dyn NativeConvert> {
    if userdata.is::<CStruct>() {
        Ok(userdata.to_pointer().cast::<CStruct>() as *const dyn NativeConvert)
    } else if userdata.is::<CArr>() {
        Ok(userdata.to_pointer().cast::<CArr>() as *const dyn NativeConvert)
    } else if userdata.is::<CPtr>() {
        Ok(userdata.to_pointer().cast::<CPtr>() as *const dyn NativeConvert)
    } else {
        c_type_helper::get_conv(userdata)
        // TODO: struct and more
    }
}

pub unsafe fn get_conv_list(table: &LuaTable) -> LuaResult<Vec<*const dyn NativeConvert>> {
    let len: usize = table.raw_len();
    let mut conv_list = Vec::<*const dyn NativeConvert>::with_capacity(len);

    for i in 0..len {
        let value: LuaValue = table.raw_get(i + 1)?;
        conv_list.push(get_conv(&get_userdata(value)?)?);
    }

    Ok(conv_list)
}

pub fn get_size(this: &LuaAnyUserData) -> LuaResult<usize> {
    if this.is::<CStruct>() {
        Ok(this.borrow::<CStruct>()?.get_size())
    } else if this.is::<CArr>() {
        Ok(this.borrow::<CArr>()?.get_size())
    } else if this.is::<CPtr>() {
        Ok(this.borrow::<CPtr>()?.get_size())
    } else {
        c_type_helper::get_size(this)
    }
}

// get libffi_type from any c-type userdata
pub fn get_middle_type(userdata: &LuaAnyUserData) -> LuaResult<Type> {
    if userdata.is::<CStruct>() {
        Ok(userdata.borrow::<CStruct>()?.get_type())
    } else if let Some(middle_type) = c_type_helper::get_middle_type(userdata)? {
        Ok(middle_type)
    } else if userdata.is::<CArr>() {
        Ok(userdata.borrow::<CArr>()?.get_type())
    } else if userdata.is::<CPtr>() {
        Ok(CPtr::get_type())
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
    if userdata.is::<CStruct>() {
        CStruct::stringify(lua, userdata)
    } else if userdata.is::<CArr>() {
        CArr::stringify(lua, userdata)
    } else if userdata.is::<CPtr>() {
        CPtr::stringify(lua, userdata)
    } else if userdata.is::<CFunc>() {
        CFunc::stringify(lua, userdata)
    } else if let Some(name) = c_type_helper::get_name(userdata)? {
        Ok(String::from(name))
    } else {
        Ok(String::from("unknown"))
    }
}

// get name tag for any c-type userdata
pub fn get_tag_name(userdata: &LuaAnyUserData) -> LuaResult<String> {
    Ok(if userdata.is::<CStruct>() {
        String::from("CStruct")
    } else if userdata.is::<CArr>() {
        String::from("CArr")
    } else if userdata.is::<CPtr>() {
        String::from("CPtr")
    } else if userdata.is::<CFunc>() {
        String::from("CFunc")
    } else if c_type_helper::is_ctype(userdata) {
        String::from("CType")
    } else {
        String::from("Unknown")
    })
}

// emulate 'print' for ctype userdata, but ctype is simplified
pub fn pretty_format(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
    if c_type_helper::is_ctype(userdata) {
        stringify(lua, userdata)
    } else {
        Ok(format!(
            "<{}({})>",
            get_tag_name(userdata)?,
            stringify(lua, userdata)?
        ))
    }
}
