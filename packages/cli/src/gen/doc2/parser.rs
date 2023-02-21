use anyhow::{Context, Result};
use full_moon::{
    ast::{
        types::{TypeFieldKey, TypeInfo},
        Stmt,
    },
    tokenizer::{TokenReference, TokenType},
};
use regex::Regex;

use super::{builder::DocItemBuilder, item::DocItem, kind::DocItemKind};

struct DocVisitorItem {
    name: String,
    comment: Option<String>,
    exported: bool,
    type_info: TypeInfo,
}

impl From<DocVisitorItem> for DocItem {
    fn from(value: DocVisitorItem) -> Self {
        let mut builder = DocItemBuilder::new()
            .with_kind(match value.type_info {
                TypeInfo::Array { .. } | TypeInfo::Table { .. } => DocItemKind::Table,
                TypeInfo::Callback { .. } => DocItemKind::Function,
                _ => unimplemented!("Support for globals that are not properties or functions is not yet implemented")
            })
            .with_name(value.name);
        if let Some(comment) = value.comment {
            builder = builder.with_children(&parse_moonwave_style_comment(&comment));
        }
        if let TypeInfo::Table { fields, .. } = value.type_info {
            for field in fields.iter() {
                if let TypeFieldKey::Name(name) = field.key() {
                    let children = find_token_moonwave_comment(name)
                        .as_deref()
                        .map(parse_moonwave_style_comment)
                        .unwrap_or_default();
                    builder = builder.with_child(
                        DocItemBuilder::new()
                            .with_kind(match field.value() {
                                TypeInfo::Callback { .. } => DocItemKind::Function,
                                _ => DocItemKind::Property,
                            })
                            .with_name(name.token().to_string())
                            .with_children(&children)
                            .build()
                            .unwrap(),
                    );
                }
            }
        }
        builder.build().unwrap()
    }
}

pub fn parse_type_definitions_into_doc_items<S>(contents: S) -> Result<Vec<DocItem>>
where
    S: AsRef<str>,
{
    let mut found_top_level_items = Vec::new();
    let ast = full_moon::parse(&cleanup_type_definitions(contents.as_ref()))
        .context("Failed to parse type definitions")?;
    for stmt in ast.nodes().stmts() {
        if let Some((declaration, token_reference)) = match stmt {
            Stmt::ExportedTypeDeclaration(exp) => {
                Some((exp.type_declaration(), exp.export_token()))
            }
            Stmt::TypeDeclaration(typ) => Some((typ, typ.type_token())),
            _ => None,
        } {
            found_top_level_items.push(DocVisitorItem {
                name: declaration.type_name().token().to_string(),
                comment: find_token_moonwave_comment(token_reference),
                exported: matches!(stmt, Stmt::ExportedTypeDeclaration(_)),
                type_info: declaration.type_definition().clone(),
            });
        }
    }
    Ok(found_top_level_items
        .drain(..)
        .filter(|item| item.exported) // NOTE: Should we include items that are not exported? Probably not ..
        .map(DocItem::from)
        .collect())
}

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

fn parse_moonwave_style_comment(comment: &str) -> Vec<DocItem> {
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

fn find_token_moonwave_comment(token: &TokenReference) -> Option<String> {
    token
        .leading_trivia()
        .filter_map(|trivia| match trivia.token_type() {
            TokenType::MultiLineComment { blocks, comment } if blocks == &1 => Some(comment),
            _ => None,
        })
        .last()
        .map(|comment| comment.trim().to_string())
}

fn cleanup_type_definitions(contents: &str) -> String {
    // TODO: Properly handle the "declare class" syntax, for now we just skip it
    let mut no_declares = contents.to_string();
    while let Some(dec) = no_declares.find("\ndeclare class") {
        let end = no_declares.find("\nend").unwrap();
        let before = &no_declares[0..dec];
        let after = &no_declares[end + 4..];
        no_declares = format!("{before}{after}");
    }
    let (regex, replacement) = (
        Regex::new(r#"declare (?P<n>\w+): "#).unwrap(),
        r#"export type $n = "#,
    );
    regex.replace_all(&no_declares, replacement).to_string()
}
