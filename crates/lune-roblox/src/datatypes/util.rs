// HACK: We round to the nearest Very Small Decimal
// to reduce writing out floating point accumulation
// errors to files (mostly relevant for xml formats)
const ROUNDING: usize = 65_536; // 2 ^ 16

pub fn round_float_decimal(value: f32) -> f32 {
    let place = ROUNDING as f32;

    // Round only the fractional part, we do not want to
    // lose any float precision in case a user for some
    // reason has very very large float numbers in files
    let whole = value.trunc();
    let fract = (value.fract() * place).round() / place;

    whole + fract
}

/**
    Converts an optional Luau number into a 16-bit integer component,
    the way Roblox constructors do it: a fraction is truncated,
    and a value outside of the 16-bit range is an error.
*/
pub fn int16_from(value: Option<f64>) -> mlua::Result<i32> {
    let value = value.unwrap_or_default().trunc();
    if value.is_finite() && value >= f64::from(i16::MIN) && value <= f64::from(i16::MAX) {
        Ok(value as i32)
    } else {
        Err(mlua::Error::RuntimeError(format!(
            "number {value} is out of range for a 16-bit integer"
        )))
    }
}
