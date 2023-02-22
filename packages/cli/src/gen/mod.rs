mod doc;
mod docs_file;
mod selene_defs;
mod wiki_dir;

pub mod definitions;

pub use docs_file::generate_from_type_definitions as generate_docs_json_from_definitions;
pub use selene_defs::generate_from_type_definitions as generate_selene_defs_from_definitions;
pub use wiki_dir::generate_from_type_definitions as generate_wiki_dir_from_definitions;

pub use self::doc::DocumentationVisitor;

pub const GENERATED_COMMENT_TAG: &str = "@generated with lune-cli";
