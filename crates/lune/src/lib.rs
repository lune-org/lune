#![allow(clippy::cargo_common_metadata)]

mod rt;

#[cfg(test)]
mod tests;

pub use crate::rt::{Runtime, RuntimeError, RuntimeResult};
