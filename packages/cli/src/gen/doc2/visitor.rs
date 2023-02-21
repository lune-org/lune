use anyhow::{Context, Result};
use full_moon::{
    ast::{types::TypeInfo, Ast, Stmt},
    tokenizer::TokenKind,
    visitors::Visitor,
};
use regex::Regex;

use super::{builder::DocItemBuilder, item::DocItem, kind::DocItemKind};

struct DocVisitorItem {
    name: String,
    comment: Option<String>,
    exported: bool,
    ast: TypeInfo,
}

impl From<DocVisitorItem> for DocItem {
    fn from(value: DocVisitorItem) -> Self {
        let mut builder = DocItemBuilder::new()
            .with_kind(DocItemKind::Global)
            .with_name(value.name);
        if let Some(comment) = value.comment {
            builder = builder.with_child(
                DocItemBuilder::new()
                    .with_kind(DocItemKind::Description)
                    .with_name("Description")
                    .with_value(comment)
                    .build()
                    .unwrap(),
            );
        }
        builder.build().unwrap()
    }
}

pub struct DocVisitor {
    pending_visitor_items: Vec<DocVisitorItem>,
}

impl DocVisitor {
    pub fn visit_type_definitions_str<S>(contents: S) -> Result<Vec<DocItem>>
    where
        S: AsRef<str>,
    {
        let mut this = Self {
            pending_visitor_items: Vec::new(),
        };
        this.visit_ast(
            &full_moon::parse(&cleanup_type_definitions(contents.as_ref()))
                .context("Failed to parse type definitions")?,
        );
        Ok(this
            .pending_visitor_items
            .drain(..)
            .filter(|item| item.exported) // NOTE: Should we include items that are not exported? Probably not ..
            .map(DocItem::from)
            .collect())
    }
}

impl Visitor for DocVisitor {
    fn visit_ast(&mut self, ast: &Ast)
    where
        Self: Sized,
    {
        for stmt in ast.nodes().stmts() {
            if let Some((declaration, leading_trivia)) = match stmt {
                Stmt::ExportedTypeDeclaration(exp) => {
                    Some((exp.type_declaration(), exp.export_token().leading_trivia()))
                }
                Stmt::TypeDeclaration(typ) => Some((typ, typ.type_token().leading_trivia())),
                _ => None,
            } {
                self.pending_visitor_items.push(DocVisitorItem {
                    name: declaration.type_name().to_string(),
                    comment: leading_trivia
                        .filter(|trivia| matches!(trivia.token_kind(), TokenKind::MultiLineComment))
                        .last()
                        .map(ToString::to_string),
                    exported: matches!(stmt, Stmt::ExportedTypeDeclaration(_)),
                    ast: declaration.type_definition().clone(),
                });
            }
        }
    }
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
