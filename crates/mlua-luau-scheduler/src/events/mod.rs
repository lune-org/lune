#![allow(unused_imports)]

mod multi;
mod once;

pub(crate) use self::multi::{MultiEvent, MultiListener};
pub(crate) use self::once::{OnceEvent, OnceListener};
