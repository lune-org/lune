
use libffi::middle::Type;
use mlua::prelude::*;

use super::{association_names::CPTR_INNER, ctype_helper, helper, method_provider};
use crate::{
    data::{ClosureData, GetFfiData, RefData, RefFlag, UNSIZED_BOUNDS},
    ffi::{
        association, libffi_helper::SIZE_OF_POINTER, FfiConvert, FfiData, FfiSignedness, FfiSize,
    },
};

const READ_CPTR_REF_FLAGS: u8 = RefFlag::Dereferenceable.value() | RefFlag::Offsetable.value();
const READ_REF_FLAGS: u8 =
    RefFlag::Offsetable.value() | RefFlag::Readable.value() | RefFlag::Writable.value();

pub struct CPtrInfo {
    inner_is_cptr: bool,
}

impl FfiSignedness for CPtrInfo {
    fn get_signedness(&self) -> bool {
        false
    }
}

impl FfiSize for CPtrInfo {
    fn get_size(&self) -> usize {
        SIZE_OF_POINTER
    }
}

impl FfiConvert for CPtrInfo {
    // Write address of RefData
    unsafe fn value_into_data(
        &self,
        _lua: &Lua,
        offset: isize,
        data_handle: &dyn FfiData,
        value: LuaValue,
    ) -> LuaResult<()> {
        let LuaValue::UserData(value_userdata) = value else {
            return Err(LuaError::external(format!(
                "Value must be a RefData, BoxData or ClosureData, got {}",
                value.type_name()
            )));
        };
        // A closure's inner pointer points at the cell holding its code pointer;
        // store the code pointer itself.
        let pointer_value = if value_userdata.is::<ClosureData>() {
            *value_userdata
                .borrow::<ClosureData>()?
                .get_inner_pointer()
                .cast::<*mut ()>()
        } else {
            value_userdata.get_ffi_data()?.get_inner_pointer()
        };
        data_handle
            .get_inner_pointer()
            .byte_offset(offset)
            .cast::<*mut ()>()
            .write_unaligned(pointer_value);
        Ok(())
    }

    // Read address, create RefData
    unsafe fn value_from_data(
        &self,
        lua: &Lua,
        offset: isize,
        data_handle: &dyn FfiData,
    ) -> LuaResult<LuaValue> {
        Ok(LuaValue::UserData(
            lua.create_userdata(RefData::new(
                data_handle
                    .get_inner_pointer()
                    .byte_offset(offset)
                    .cast::<*mut ()>()
                    .read_unaligned(),
                if self.inner_is_cptr {
                    READ_CPTR_REF_FLAGS
                } else {
                    READ_REF_FLAGS
                },
                UNSIZED_BOUNDS,
            ))?,
        ))
    }

    // Copy Address
    unsafe fn copy_data(
        &self,
        _lua: &Lua,
        dst_offset: isize,
        src_offset: isize,
        dst: &dyn FfiData,
        src: &dyn FfiData,
    ) -> LuaResult<()> {
        let pointer_value = src
            .get_inner_pointer()
            .byte_offset(src_offset)
            .cast::<*mut ()>()
            .read_unaligned();
        dst.get_inner_pointer()
            .byte_offset(dst_offset)
            .cast::<*mut ()>()
            .write_unaligned(pointer_value);
        Ok(())
    }
}

impl CPtrInfo {
    // Create pointer type with '.inner' field
    // inner can be CArr, CType or CStruct
    pub fn from_userdata(
        lua: &Lua,
        inner: &LuaAnyUserData,
    ) -> LuaResult<LuaAnyUserData> {
        let value = lua.create_userdata(Self {
            inner_is_cptr: inner.is::<CPtrInfo>(),
        })?;

        association::set(lua, CPTR_INNER, &value, inner)?;

        Ok(value)
    }

    // Stringify CPtr with inner ctype
    pub fn stringify(lua: &Lua, userdata: &LuaAnyUserData) -> LuaResult<String> {
        if let LuaValue::UserData(inner_userdata) = userdata.get("inner")? {
            let pretty_formatted = helper::pretty_format(lua, &inner_userdata)?;
            Ok(if ctype_helper::is_ctype(&inner_userdata) {
                pretty_formatted
            } else {
                format!(" {pretty_formatted} ")
            })
        } else {
            Err(LuaError::external("Failed to retrieve inner type"))
        }
    }

    // Return void*
    pub fn get_middle_type() -> Type {
        Type::pointer()
    }
}

impl LuaUserData for CPtrInfo {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_lua, _this| Ok(SIZE_OF_POINTER));
        fields.add_field_function_get("inner", |lua, this| {
            association::get(lua, CPTR_INNER, this)?
                .ok_or_else(|| LuaError::external("Failed to retrieve inner type"))
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // Subtype
        method_provider::provide_ptr(methods);
        method_provider::provide_arr(methods);

        // ToString
        method_provider::provide_to_string(methods);

        // Realize: create a pointer-sized box holding an address
        method_provider::provide_box(methods);

        methods.add_method(
            "readRef",
            |lua,
             this,
             (target, offset, ref_data): (
                LuaAnyUserData,
                Option<isize>,
                Option<LuaAnyUserData>,
            )| unsafe {
                if let Some(ref_userdata) = ref_data {
                    if !ref_userdata.is::<RefData>() {
                        return Err(LuaError::external(
                            "Argument 'ref' must be a RefData when provided",
                        ));
                    }
                    RefData::update(
                        lua,
                        ref_userdata.clone(),
                        target
                            .get_ffi_data()?
                            .get_inner_pointer()
                            .byte_offset(offset.unwrap_or(0))
                            .cast::<*mut ()>()
                            .read_unaligned(),
                        if this.inner_is_cptr {
                            READ_CPTR_REF_FLAGS
                        } else {
                            READ_REF_FLAGS
                        },
                        UNSIZED_BOUNDS,
                    )?;
                    Ok(LuaValue::UserData(ref_userdata))
                } else {
                    this.value_from_data(lua, offset.unwrap_or(0), &target.get_ffi_data()?)
                }
            },
        );
        methods.add_method(
            "writeRef",
            |lua, this, (target, value, offset): (LuaAnyUserData, LuaValue, Option<isize>)| unsafe {
                this.value_into_data(lua, offset.unwrap_or(0), &target.get_ffi_data()?, value)
            },
        );
    }
}
