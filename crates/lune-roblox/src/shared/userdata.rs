#![allow(clippy::missing_errors_doc)]

use std::{any::type_name, cell::RefCell, fmt, ops};

use mlua::prelude::*;

// Utility functions

type ListWriter = dyn Fn(&mut fmt::Formatter<'_>, bool, &str) -> fmt::Result;

#[must_use]
pub fn make_list_writer() -> Box<ListWriter> {
    let first = RefCell::new(true);
    Box::new(move |f, flag, literal| {
        if flag {
            if first.take() {
                write!(f, "{literal}")?;
            } else {
                write!(f, ", {literal}")?;
            }
        }
        Ok::<_, fmt::Error>(())
    })
}

/*
    Userdata metamethod implementations

    Note that many of these return [`LuaResult`] even though they don't
    return any errors - this is for consistency reasons and to make it
    easier to add these blanket implementations to [`LuaUserData`] impls.
*/

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

pub fn userdata_impl_add<D>(_: &Lua, datatype: &D, value: LuaUserDataRef<D>) -> LuaResult<D>
where
    D: LuaUserData + ops::Add<Output = D> + Copy,
{
    Ok(*datatype + *value)
}

pub fn userdata_impl_sub<D>(_: &Lua, datatype: &D, value: LuaUserDataRef<D>) -> LuaResult<D>
where
    D: LuaUserData + ops::Sub<Output = D> + Copy,
{
    Ok(*datatype - *value)
}

fn borrow_datatype<D>(value: &LuaValue) -> Option<D>
where
    D: LuaUserData + Copy + 'static,
{
    if let LuaValue::UserData(ud) = value {
        ud.borrow::<D>().ok().map(|d| *d)
    } else {
        None
    }
}

fn mul_div_error<D>(lhs: &LuaValue, rhs: &LuaValue) -> LuaError {
    LuaError::FromLuaConversionError {
        from: rhs.type_name(),
        to: type_name::<D>().to_string(),
        message: Some(format!(
            "Expected {} or number, got {} and {}",
            type_name::<D>(),
            lhs.type_name(),
            rhs.type_name()
        )),
    }
}

// NOTE: Multiplication is registered as a meta *function* (not method) so that
// scalar-on-the-left (`2 * vector`) works as well as scalar-on-the-right - a
// meta method would bind `self` to the first operand, which fails when it is
// the scalar rather than the userdata.
pub fn userdata_impl_mul_f32<D>(_: &Lua, (lhs, rhs): (LuaValue, LuaValue)) -> LuaResult<D>
where
    D: LuaUserData + ops::Mul<D, Output = D> + ops::Mul<f32, Output = D> + Copy + 'static,
{
    let scalar = |v: &LuaValue| match v {
        LuaValue::Number(n) => Some(*n as f32),
        LuaValue::Integer(i) => Some(*i as f32),
        _ => None,
    };
    match (borrow_datatype::<D>(&lhs), borrow_datatype::<D>(&rhs)) {
        (Some(a), Some(b)) => Some(a * b),
        (Some(a), None) => scalar(&rhs).map(|s| a * s),
        (None, Some(b)) => scalar(&lhs).map(|s| b * s),
        (None, None) => None,
    }
    .ok_or_else(|| mul_div_error::<D>(&lhs, &rhs))
}

pub fn userdata_impl_mul_i32<D>(_: &Lua, (lhs, rhs): (LuaValue, LuaValue)) -> LuaResult<D>
where
    D: LuaUserData + ops::Mul<D, Output = D> + ops::Mul<i32, Output = D> + Copy + 'static,
{
    let scalar = |v: &LuaValue| match v {
        LuaValue::Number(n) => Some(*n as i32),
        LuaValue::Integer(i) => Some(*i as i32),
        _ => None,
    };
    match (borrow_datatype::<D>(&lhs), borrow_datatype::<D>(&rhs)) {
        (Some(a), Some(b)) => Some(a * b),
        (Some(a), None) => scalar(&rhs).map(|s| a * s),
        (None, Some(b)) => scalar(&lhs).map(|s| b * s),
        (None, None) => None,
    }
    .ok_or_else(|| mul_div_error::<D>(&lhs, &rhs))
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
    }
    Err(LuaError::FromLuaConversionError {
        from: rhs.type_name(),
        to: type_name::<D>().to_string(),
        message: Some(format!(
            "Expected {} or number, got {}",
            type_name::<D>(),
            rhs.type_name()
        )),
    })
}

pub trait IDiv<Rhs = Self> {
    type Output;
    #[must_use]
    fn idiv(self, rhs: Rhs) -> Self::Output;
}

pub fn userdata_impl_idiv_f32<D>(_: &Lua, datatype: &D, rhs: LuaValue) -> LuaResult<D>
where
    D: LuaUserData + IDiv<D, Output = D> + IDiv<f32, Output = D> + Copy + 'static,
{
    match &rhs {
        LuaValue::Number(n) => return Ok(datatype.idiv(*n as f32)),
        LuaValue::Integer(i) => return Ok(datatype.idiv(*i as f32)),
        LuaValue::UserData(ud) => {
            if let Ok(vec) = ud.borrow::<D>() {
                return Ok(datatype.idiv(*vec));
            }
        }
        _ => {}
    }
    Err(LuaError::FromLuaConversionError {
        from: rhs.type_name(),
        to: type_name::<D>().to_string(),
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
        LuaValue::Integer(i) => return Ok(*datatype / *i as i32),
        LuaValue::UserData(ud) => {
            if let Ok(vec) = ud.borrow::<D>() {
                return Ok(*datatype / *vec);
            }
        }
        _ => {}
    }
    Err(LuaError::FromLuaConversionError {
        from: rhs.type_name(),
        to: type_name::<D>().to_string(),
        message: Some(format!(
            "Expected {} or number, got {}",
            type_name::<D>(),
            rhs.type_name()
        )),
    })
}
