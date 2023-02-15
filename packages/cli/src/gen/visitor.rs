use full_moon::{
    ast::types::{ExportedTypeDeclaration, TypeField, TypeFieldKey},
    tokenizer::{Token, TokenType},
    visitors::Visitor,
};
use regex::Regex;

use super::{
    doc::{DocsFunction, DocsFunctionParamLink, DocsGlobal, DocsParam, DocsReturn},
    tag::{DocsTag, DocsTagKind, DocsTagList},
};

#[derive(Debug, Clone)]
pub struct DocumentationVisitor {
    pub globals: Vec<(String, DocsGlobal)>,
    pub functions: Vec<(String, DocsFunction)>,
    pub params: Vec<(String, DocsParam)>,
    pub returns: Vec<(String, DocsReturn)>,
    tag_regex: Regex,
}

impl DocumentationVisitor {
    pub fn new() -> Self {
        let tag_regex = Regex::new(r#"^@(\S+)\s+(\S+)(.*)$"#).unwrap();
        Self {
            globals: vec![],
            functions: vec![],
            params: vec![],
            returns: vec![],
            tag_regex,
        }
    }

    pub fn parse_moonwave_style_tag(&self, line: &str) -> Option<DocsTag> {
        if self.tag_regex.is_match(line) {
            let captures = self.tag_regex.captures(line).unwrap();
            let tag_kind = captures.get(1).unwrap().as_str();
            let tag_name = captures.get(2).unwrap().as_str();
            let tag_contents = captures.get(3).unwrap().as_str();
            Some(DocsTag {
                kind: DocsTagKind::parse(tag_kind).unwrap(),
                name: tag_name.to_string(),
                contents: tag_contents.to_string(),
            })
        } else {
            None
        }
    }

    pub fn parse_moonwave_style_comment(&self, comment: &str) -> (String, DocsTagList) {
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
        let mut doc_lines = Vec::new();
        let mut doc_tags = DocsTagList::new();
        for line in unindented_lines {
            if let Some(tag) = self.parse_moonwave_style_tag(line) {
                doc_tags.push(tag);
            } else {
                doc_lines.push(line);
            }
        }
        (doc_lines.join("\n").trim().to_owned(), doc_tags)
    }

    fn extract_moonwave_comment(&mut self, token: &Token) -> Option<(String, DocsTagList)> {
        if let TokenType::MultiLineComment { comment, .. } = token.token_type() {
            let (doc, tags) = self.parse_moonwave_style_comment(comment);
            if doc.is_empty() && tags.is_empty() {
                None
            } else {
                Some((doc, tags))
            }
        } else {
            None
        }
    }
}

impl Visitor for DocumentationVisitor {
    fn visit_exported_type_declaration(&mut self, node: &ExportedTypeDeclaration) {
        for token in node.export_token().leading_trivia() {
            if let Some((doc, mut tags)) = self.extract_moonwave_comment(token) {
                if tags.contains(DocsTagKind::Class) {
                    self.globals.push((
                        node.type_declaration().type_name().token().to_string(),
                        DocsGlobal {
                            documentation: doc,
                            ..Default::default()
                        },
                    ));
                    break;
                }
            }
        }
    }

    fn visit_type_field(&mut self, node: &TypeField) {
        // Parse out names, moonwave comments from the ast
        let mut parsed_data = Vec::new();
        if let TypeFieldKey::Name(name) = node.key() {
            for token in name.leading_trivia() {
                if let Some((doc, mut tags)) = self.extract_moonwave_comment(token) {
                    if let Some(within) = tags.find(DocsTagKind::Within).map(ToOwned::to_owned) {
                        parsed_data.push((within.name, name, doc, tags));
                    }
                }
            }
        }
        for (global_name, name, doc, tags) in parsed_data {
            // Find the global definition, which is guaranteed to
            // be visited and parsed before its inner members, and
            // add a ref to the found function / member to it
            let name = name.token().to_string();
            for (name, global) in &mut self.globals {
                if name == &global_name {
                    global.keys.insert(name.clone(), name.clone());
                }
            }
            // Look through tags to find and create doc params and returns
            let mut param_links = Vec::new();
            let mut return_links = Vec::new();
            for tag in tags {
                match tag.kind {
                    DocsTagKind::Param => {
                        let idx_string = param_links.len().to_string();
                        self.params.push((
                            idx_string.clone(),
                            DocsParam {
                                global_name: global_name.clone(),
                                function_name: name.clone(),
                                documentation: tag.contents.trim().to_owned(),
                            },
                        ));
                        param_links.push(DocsFunctionParamLink {
                            name: tag.name.clone(),
                            documentation: idx_string.clone(),
                        });
                    }
                    DocsTagKind::Return => {
                        // NOTE: Returns don't have names but we still parse
                        // them as such, so we should concat name & contents
                        let doc = format!("{} {}", tag.name.trim(), tag.contents.trim());
                        let idx_string = return_links.len().to_string();
                        self.returns.push((
                            idx_string.clone(),
                            DocsReturn {
                                global_name: global_name.clone(),
                                function_name: name.clone(),
                                documentation: doc,
                            },
                        ));
                        return_links.push(idx_string.clone());
                    }
                    _ => {}
                }
            }
            // Finally, add our complete doc
            // function with links into the list
            self.functions.push((
                name,
                DocsFunction {
                    global_name,
                    documentation: doc,
                    params: param_links,
                    returns: return_links,
                    ..Default::default()
                },
            ));
        }
    }
}
