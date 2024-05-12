pub(crate) use rbx_dom_weak::types::{Variant as DomValue, VariantType as DomType};

pub mod attributes;
pub mod conversion;
pub mod extension;
pub mod result;
pub mod types;

mod util;

use result::*;

pub use crate::shared::userdata::*;
