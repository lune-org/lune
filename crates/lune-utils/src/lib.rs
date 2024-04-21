#![allow(clippy::cargo_common_metadata)]

mod table_builder;
mod version_string;

pub use self::table_builder::TableBuilder;
pub use self::version_string::get_version_string;
