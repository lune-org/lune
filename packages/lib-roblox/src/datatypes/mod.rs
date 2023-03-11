pub(crate) use rbx_dom_weak::types::{Variant as RbxVariant, VariantType as RbxVariantType};

mod conversion;
mod extension;
mod result;
mod shared;

pub mod types;

use conversion::*;
use extension::*;
use result::*;
use shared::*;
