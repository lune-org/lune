mod doc;
mod docs_file;
mod wiki_dir;

pub use docs_file::generate_from_type_definitions as generate_docs_json_from_definitions;
pub use wiki_dir::generate_from_type_definitions as generate_wiki_dir_from_definitions;

pub use self::doc::DocumentationVisitor;

pub const GENERATED_COMMENT_TAG: &str = "@generated with lune-cli";
