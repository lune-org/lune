#![allow(clippy::cargo_common_metadata)]

mod rt;

// TODO: Remove this in 0.9.0 since it is now available as a separate crate!
#[cfg(feature = "std-roblox")]
pub use lune_roblox as roblox;

#[cfg(test)]
mod tests;

pub use crate::rt::{Runtime, RuntimeError, RuntimeResult};
