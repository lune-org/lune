use regex::Regex;

use super::{builder::DocItemBuilder, item::DocItem, kind::DocItemKind};

fn should_separate_tag_meta(tag_kind: &str) -> bool {
    matches!(tag_kind.trim().to_ascii_lowercase().as_ref(), "param")
}

fn parse_moonwave_style_tag(line: &str) -> Option<DocItem> {
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
            let mut builder = DocItemBuilder::new()
                .with_kind(DocItemKind::Tag)
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

pub(super) fn parse_moonwave_style_comment(comment: &str) -> Vec<DocItem> {
    let lines = comment.lines().map(str::trim).collect::<Vec<_>>();
    let indent_len = lines.iter().fold(usize::MAX, |acc, line| {
        let first = line.chars().enumerate().find_map(|(idx, ch)| {
            if ch.is_alphanumeric() {
                Some(idx)
            } else {
                None
            }
        });
        if let Some(first_alphanumeric) = first {
            if first_alphanumeric > 0 {
                acc.min(first_alphanumeric - 1)
            } else {
                0
            }
        } else {
            acc
        }
    });
    let unindented_lines = lines.iter().map(|line| &line[indent_len..]);
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
            DocItemBuilder::new()
                .with_kind(DocItemKind::Description)
                .with_value(doc_lines.join("\n").trim())
                .build()
                .unwrap(),
        );
    }
    doc_items
}
