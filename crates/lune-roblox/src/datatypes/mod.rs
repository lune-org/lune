pub(crate) use rbx_dom_weak::types::{Variant as DomValue, VariantType as DomType};

pub mod extension;

#[cfg(feature = "mlua")]
pub mod attributes;
#[cfg(feature = "mlua")]
pub mod conversion;
#[cfg(feature = "mlua")]
pub mod result;
#[cfg(feature = "mlua")]
pub mod types;

#[cfg(feature = "mlua")]
mod util;

#[cfg(feature = "mlua")]
use result::*;

#[cfg(feature = "mlua")]
pub use crate::shared::userdata::*;
