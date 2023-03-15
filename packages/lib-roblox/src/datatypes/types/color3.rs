use core::fmt;
use std::ops;

use glam::Vec3;
use mlua::prelude::*;
use rbx_dom_weak::types::{Color3 as RbxColor3, Color3uint8 as RbxColor3uint8};

use super::super::*;

/**
    An implementation of the [Color3](https://create.roblox.com/docs/reference/engine/datatypes/Color3) Roblox datatype.

    This implements all documented properties, methods & constructors of the Color3 class as of March 2023.

    It also implements math operations for addition, subtraction, multiplication, and division,
    all of which are suspiciously missing from the Roblox implementation of the Color3 datatype.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color3 {
    pub(crate) r: f32,
    pub(crate) g: f32,
    pub(crate) b: f32,
}

impl Color3 {
    pub(crate) fn make_table(lua: &Lua, datatype_table: &LuaTable) -> LuaResult<()> {
        datatype_table.set(
            "new",
            lua.create_function(|_, (r, g, b): (Option<f32>, Option<f32>, Option<f32>)| {
                Ok(Color3 {
                    r: r.unwrap_or_default(),
                    g: g.unwrap_or_default(),
                    b: b.unwrap_or_default(),
                })
            })?,
        )?;
        datatype_table.set(
            "fromRGB",
            lua.create_function(|_, (r, g, b): (Option<u8>, Option<u8>, Option<u8>)| {
                Ok(Color3 {
                    r: (r.unwrap_or_default() as f32) / 255f32,
                    g: (g.unwrap_or_default() as f32) / 255f32,
                    b: (b.unwrap_or_default() as f32) / 255f32,
                })
            })?,
        )?;
        datatype_table.set(
            "fromHSV",
            lua.create_function(|_, (h, s, v): (f32, f32, f32)| {
                // https://axonflux.com/handy-rgb-to-hsl-and-rgb-to-hsv-color-model-c
                let i = (h * 6.0).floor();
                let f = h * 6.0 - i;
                let p = v * (1.0 - s);
                let q = v * (1.0 - f * s);
                let t = v * (1.0 - (1.0 - f) * s);

                let (r, g, b) = match (i % 6.0) as u8 {
                    0 => (v, t, p),
                    1 => (q, v, p),
                    2 => (p, v, t),
                    3 => (p, q, v),
                    4 => (t, p, v),
                    5 => (v, p, q),
                    _ => unreachable!(),
                };

                Ok(Color3 { r, g, b })
            })?,
        )?;
        datatype_table.set(
            "fromHex",
            lua.create_function(|_, hex: String| {
                let trimmed = hex.trim_start_matches('#').to_ascii_uppercase();
                let chars = if trimmed.len() == 3 {
                    (
                        u8::from_str_radix(&trimmed[..1].repeat(2), 16),
                        u8::from_str_radix(&trimmed[1..2].repeat(2), 16),
                        u8::from_str_radix(&trimmed[2..3].repeat(2), 16),
                    )
                } else if trimmed.len() == 6 {
                    (
                        u8::from_str_radix(&trimmed[..2], 16),
                        u8::from_str_radix(&trimmed[2..4], 16),
                        u8::from_str_radix(&trimmed[4..6], 16),
                    )
                } else {
                    return Err(LuaError::RuntimeError(format!(
                        "Hex color string must be 3 or 6 characters long, got {} character{}",
                        trimmed.len(),
                        if trimmed.len() == 1 { "" } else { "s" }
                    )));
                };
                match chars {
                    (Ok(r), Ok(g), Ok(b)) => Ok(Color3 {
                        r: (r as f32) / 255f32,
                        g: (g as f32) / 255f32,
                        b: (b as f32) / 255f32,
                    }),
                    _ => Err(LuaError::RuntimeError(format!(
                        "Hex color string '{}' contains invalid character",
                        trimmed
                    ))),
                }
            })?,
        )?;
        Ok(())
    }
}

impl LuaUserData for Color3 {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("R", |_, this| Ok(this.r));
        fields.add_field_method_get("G", |_, this| Ok(this.g));
        fields.add_field_method_get("B", |_, this| Ok(this.b));
    }

    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // Methods
        methods.add_method("Lerp", |_, this, (rhs, alpha): (Color3, f32)| {
            let v3_this = Vec3::new(this.r, this.g, this.b);
            let v3_rhs = Vec3::new(rhs.r, rhs.g, rhs.b);
            let v3 = v3_this.lerp(v3_rhs, alpha);
            Ok(Color3 {
                r: v3.x,
                g: v3.y,
                b: v3.z,
            })
        });
        methods.add_method("ToHSV", |_, this, ()| {
            // https://axonflux.com/handy-rgb-to-hsl-and-rgb-to-hsv-color-model-c
            let (r, g, b) = (this.r, this.g, this.b);
            let min = r.min(g).min(b);
            let max = r.max(g).max(b);
            let diff = max - min;
            let hue = match max {
                max if max == min => 0.0,
                max if max == this.r => (g - b) + diff * (if g < b { 6.0 } else { 0.0 }),
                max if max == this.g => (b - r) + diff * 2.0,
                max if max == this.b => (r - g) + diff * 4.0,
                _ => unreachable!(),
            };
            let hue = hue / 6.0 * diff;
            let sat = if max == 0.0 { 0.0 } else { diff / max };
            let sat = sat.clamp(0.0, 1.0);
            Ok((hue, sat, max))
        });
        methods.add_method("ToHex", |_, this, ()| {
            Ok(format!(
                "{:02X}{:02X}{:02X}",
                this.r.clamp(u8::MIN as f32, u8::MAX as f32) as u8,
                this.g.clamp(u8::MIN as f32, u8::MAX as f32) as u8,
                this.b.clamp(u8::MIN as f32, u8::MAX as f32) as u8,
            ))
        });
        // Metamethods
        methods.add_meta_method(LuaMetaMethod::Eq, userdata_impl_eq);
        methods.add_meta_method(LuaMetaMethod::ToString, userdata_impl_to_string);
        methods.add_meta_method(LuaMetaMethod::Unm, userdata_impl_unm);
        methods.add_meta_method(LuaMetaMethod::Add, userdata_impl_add);
        methods.add_meta_method(LuaMetaMethod::Sub, userdata_impl_sub);
        methods.add_meta_method(LuaMetaMethod::Mul, userdata_impl_mul_f32);
        methods.add_meta_method(LuaMetaMethod::Div, userdata_impl_div_f32);
    }
}

impl Default for Color3 {
    fn default() -> Self {
        Self {
            r: 0f32,
            g: 0f32,
            b: 0f32,
        }
    }
}

impl fmt::Display for Color3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.r, self.g, self.b)
    }
}

impl ops::Neg for Color3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Color3 {
            r: -self.r,
            g: -self.g,
            b: -self.b,
        }
    }
}

impl ops::Add for Color3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Color3 {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl ops::Sub for Color3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Color3 {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
        }
    }
}

impl ops::Mul for Color3 {
    type Output = Color3;
    fn mul(self, rhs: Self) -> Self::Output {
        Color3 {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl ops::Mul<f32> for Color3 {
    type Output = Color3;
    fn mul(self, rhs: f32) -> Self::Output {
        Color3 {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl ops::Div for Color3 {
    type Output = Color3;
    fn div(self, rhs: Self) -> Self::Output {
        Color3 {
            r: self.r / rhs.r,
            g: self.g / rhs.g,
            b: self.b / rhs.b,
        }
    }
}

impl ops::Div<f32> for Color3 {
    type Output = Color3;
    fn div(self, rhs: f32) -> Self::Output {
        Color3 {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}

impl From<RbxColor3> for Color3 {
    fn from(v: RbxColor3) -> Self {
        Self {
            r: v.r,
            g: v.g,
            b: v.b,
        }
    }
}

impl From<Color3> for RbxColor3 {
    fn from(v: Color3) -> Self {
        Self {
            r: v.r,
            g: v.g,
            b: v.b,
        }
    }
}

impl From<RbxColor3uint8> for Color3 {
    fn from(v: RbxColor3uint8) -> Self {
        Self {
            r: (v.r as f32) / 255f32,
            g: (v.g as f32) / 255f32,
            b: (v.b as f32) / 255f32,
        }
    }
}

impl From<Color3> for RbxColor3uint8 {
    fn from(v: Color3) -> Self {
        Self {
            r: v.r.clamp(u8::MIN as f32, u8::MAX as f32) as u8,
            g: v.g.clamp(u8::MIN as f32, u8::MAX as f32) as u8,
            b: v.b.clamp(u8::MIN as f32, u8::MAX as f32) as u8,
        }
    }
}
