#![allow(clippy::cargo_common_metadata)]

mod rt;

#[cfg(feature = "std-roblox")]
pub use lune_roblox as roblox;

#[cfg(test)]
mod tests;

pub use crate::rt::{Runtime, RuntimeError, RuntimeResult};
