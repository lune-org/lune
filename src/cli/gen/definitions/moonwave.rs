use regex::Regex;

use super::{builder::DefinitionsItemBuilder, item::DefinitionsItem, kind::DefinitionsItemKind};

fn should_separate_tag_meta(tag_kind: &str) -> bool {
    matches!(tag_kind.trim().to_ascii_lowercase().as_ref(), "param")
}

fn parse_moonwave_style_tag(line: &str) -> Option<DefinitionsItem> {
    let tag_regex = Regex::new(r#"^@(\S+)\s*(.*)$"#).unwrap();
    if tag_regex.is_match(line) {
        let captures = tag_regex.captures(line).unwrap();
        let tag_kind = captures.get(1).unwrap().as_str();
        let tag_rest = captures.get(2).unwrap().as_str();
        let mut tag_words = tag_rest.split_whitespace().collect::<Vec<_>>();
        let tag_name = if !tag_words.is_empty() && should_separate_tag_meta(tag_kind) {
            tag_words.remove(0).to_string()
        } else {
            String::new()
        };
        let tag_contents = tag_words.join(" ");
        if tag_kind.is_empty() {
            None
        } else {
            let mut builder = DefinitionsItemBuilder::new()
                .with_kind(DefinitionsItemKind::Tag)
                .with_name(tag_kind);
            if !tag_name.is_empty() {
                builder = builder.with_meta(tag_name);
            }
            if !tag_contents.is_empty() {
                builder = builder.with_value(tag_contents);
            }
            Some(builder.build().unwrap())
        }
    } else {
        None
    }
}

pub(super) fn parse_moonwave_style_comment(comment: &str) -> Vec<DefinitionsItem> {
    let no_tabs = comment.replace('\t', "    ");
    let lines = no_tabs.split('\n').collect::<Vec<_>>();
    let indent_len =
        lines.iter().fold(usize::MAX, |acc, line| {
            let first = line.chars().enumerate().find_map(|(idx, ch)| {
                if ch.is_whitespace() {
                    None
                } else {
                    Some(idx)
                }
            });
            if let Some(first_non_whitespace) = first {
                acc.min(first_non_whitespace)
            } else {
                acc
            }
        });
    let unindented_lines = lines.iter().map(|line| {
        if line.chars().any(|c| !c.is_whitespace()) {
            &line[indent_len..]
        } else {
            line
        }
    });
    let mut doc_items = Vec::new();
    let mut doc_lines = Vec::new();
    for line in unindented_lines {
        if let Some(tag) = parse_moonwave_style_tag(line) {
            doc_items.push(tag);
        } else {
            doc_lines.push(line);
        }
    }
    if !doc_lines.is_empty() {
        doc_items.push(
            DefinitionsItemBuilder::new()
                .with_kind(DefinitionsItemKind::Description)
                .with_value(doc_lines.join("\n"))
                .build()
                .unwrap(),
        );
    }
    doc_items
}
