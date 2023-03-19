use std::{any::type_name, cell::RefCell, fmt, ops};

use mlua::prelude::*;

// Utility functions

type ListWriter = dyn Fn(&mut fmt::Formatter<'_>, bool, &str) -> fmt::Result;
pub fn make_list_writer() -> Box<ListWriter> {
    let first = RefCell::new(true);
    Box::new(move |f, flag, literal| {
        if flag {
            if first.take() {
                write!(f, "{}", literal)?;
            } else {
                write!(f, ", {}", literal)?;
            }
        }
        Ok::<_, fmt::Error>(())
    })
}

// Userdata metamethod implementations

pub fn userdata_impl_to_string<D>(_: &Lua, datatype: &D, _: ()) -> LuaResult<String>
where
    D: LuaUserData + ToString + 'static,
{
    Ok(datatype.to_string())
}

pub fn userdata_impl_eq<D>(_: &Lua, datatype: &D, value: LuaValue) -> LuaResult<bool>
where
    D: LuaUserData + PartialEq + 'static,
{
    if let LuaValue::UserData(ud) = value {
        if let Ok(value_as_datatype) = ud.borrow::<D>() {
            Ok(*datatype == *value_as_datatype)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

pub fn userdata_impl_unm<D>(_: &Lua, datatype: &D, _: ()) -> LuaResult<D>
where
    D: LuaUserData + ops::Neg<Output = D> + Copy,
{
    Ok(-*datatype)
}

pub fn userdata_impl_add<D>(_: &Lua, datatype: &D, value: D) -> LuaResult<D>
where
    D: LuaUserData + ops::Add<Output = D> + Copy,
{
    Ok(*datatype + value)
}

pub fn userdata_impl_sub<D>(_: &Lua, datatype: &D, value: D) -> LuaResult<D>
where
    D: LuaUserData + ops::Sub<Output = D> + Copy,
{
    Ok(*datatype - value)
}

pub fn userdata_impl_mul_f32<D>(_: &Lua, datatype: &D, rhs: LuaValue) -> LuaResult<D>
where
    D: LuaUserData + ops::Mul<D, Output = D> + ops::Mul<f32, Output = D> + Copy + 'static,
{
    match &rhs {
        LuaValue::Number(n) => return Ok(*datatype * *n as f32),
        LuaValue::Integer(i) => return Ok(*datatype * *i as f32),
        LuaValue::UserData(ud) => {
            if let Ok(vec) = ud.borrow::<D>() {
                return Ok(*datatype * *vec);
            }
        }
        _ => {}
    };
    Err(LuaError::FromLuaConversionError {
        from: rhs.type_name(),
        to: type_name::<D>(),
        message: Some(format!(
            "Expected {} or number, got {}",
            type_name::<D>(),
            rhs.type_name()
        )),
    })
}

pub fn userdata_impl_mul_i32<D>(_: &Lua, datatype: &D, rhs: LuaValue) -> LuaResult<D>
where
    D: LuaUserData + ops::Mul<D, Output = D> + ops::Mul<i32, Output = D> + Copy + 'static,
{
    match &rhs {
        LuaValue::Number(n) => return Ok(*datatype * *n as i32),
        LuaValue::Integer(i) => return Ok(*datatype * *i),
        LuaValue::UserData(ud) => {
            if let Ok(vec) = ud.borrow::<D>() {
                return Ok(*datatype * *vec);
            }
        }
        _ => {}
    };
    Err(LuaError::FromLuaConversionError {
        from: rhs.type_name(),
        to: type_name::<D>(),
        message: Some(format!(
            "Expected {} or number, got {}",
            type_name::<D>(),
            rhs.type_name()
        )),
    })
}

pub fn userdata_impl_div_f32<D>(_: &Lua, datatype: &D, rhs: LuaValue) -> LuaResult<D>
where
    D: LuaUserData + ops::Div<D, Output = D> + ops::Div<f32, Output = D> + Copy + 'static,
{
    match &rhs {
        LuaValue::Number(n) => return Ok(*datatype / *n as f32),
        LuaValue::Integer(i) => return Ok(*datatype / *i as f32),
        LuaValue::UserData(ud) => {
            if let Ok(vec) = ud.borrow::<D>() {
                return Ok(*datatype / *vec);
            }
        }
        _ => {}
    };
    Err(LuaError::FromLuaConversionError {
        from: rhs.type_name(),
        to: type_name::<D>(),
        message: Some(format!(
            "Expected {} or number, got {}",
            type_name::<D>(),
            rhs.type_name()
        )),
    })
}

pub fn userdata_impl_div_i32<D>(_: &Lua, datatype: &D, rhs: LuaValue) -> LuaResult<D>
where
    D: LuaUserData + ops::Div<D, Output = D> + ops::Div<i32, Output = D> + Copy + 'static,
{
    match &rhs {
        LuaValue::Number(n) => return Ok(*datatype / *n as i32),
        LuaValue::Integer(i) => return Ok(*datatype / *i),
        LuaValue::UserData(ud) => {
            if let Ok(vec) = ud.borrow::<D>() {
                return Ok(*datatype / *vec);
            }
        }
        _ => {}
    };
    Err(LuaError::FromLuaConversionError {
        from: rhs.type_name(),
        to: type_name::<D>(),
        message: Some(format!(
            "Expected {} or number, got {}",
            type_name::<D>(),
            rhs.type_name()
        )),
    })
}
