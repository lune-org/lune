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
