use include_dir::Dir;
use regex::Regex;

mod docs_file;
mod luau_defs;
mod selene_defs;
mod wiki_dir;

pub mod definitions;

pub use docs_file::generate_from_type_definitions as generate_docs_json_from_definitions;
pub use luau_defs::generate_from_type_definitions as generate_luau_defs_from_definitions;
pub use selene_defs::generate_from_type_definitions as generate_selene_defs_from_definitions;
pub use wiki_dir::generate_from_type_definitions as generate_wiki_dir_from_definitions;

pub fn generate_typedefs_file_from_dir(dir: &Dir<'_>) -> String {
    let mut result = String::new();

    for entry in dir.find("*.luau").unwrap() {
        let entry_file = entry.as_file().unwrap();
        let entry_name = entry_file.path().file_name().unwrap().to_string_lossy();

        if entry_name.contains("Globals") {
            continue;
        }

        let typedef_name = entry_name.trim_end_matches(".luau");
        let typedef_contents = entry_file.contents_utf8().unwrap().to_string().replace(
            &format!("export type {typedef_name} = "),
            &format!("declare {}: ", typedef_name.to_ascii_lowercase()),
        );

        if !result.is_empty() {
            result.push_str(&"\n".repeat(10));
        }

        result.push_str(&typedef_contents);
    }

    let globals_contents = dir
        .get_file("Globals.luau")
        .unwrap()
        .contents_utf8()
        .unwrap();

    let regex_export_to_declare = Regex::new(r#"export type (\w+) = "#).unwrap();
    let regexed_globals = regex_export_to_declare.replace_all(globals_contents, "declare $1: ");

    result.push_str(&"\n".repeat(10));
    result.push_str(&regexed_globals);

    result
}
