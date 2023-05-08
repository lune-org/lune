use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{Context, Result};
use full_moon::{
    ast::{
        types::{TypeFieldKey, TypeInfo},
        Stmt,
    },
    tokenizer::{TokenReference, TokenType},
};

use super::{
    builder::DefinitionsItemBuilder, item::DefinitionsItem, moonwave::parse_moonwave_style_comment,
    type_info_ext::TypeInfoExt, DefinitionsItemKind,
};

#[derive(Debug, Clone)]
struct DefinitionsParserItem {
    name: String,
    comment: Option<String>,
    type_info: TypeInfo,
}

#[derive(Debug, Clone)]
pub struct DefinitionsParser {
    found_top_level_items: BTreeMap<String, DefinitionsParserItem>,
    found_top_level_types: HashMap<String, TypeInfo>,
    found_top_level_comments: HashMap<String, Option<String>>,
    found_top_level_exports: Vec<String>,
}

impl DefinitionsParser {
    pub fn new() -> Self {
        Self {
            found_top_level_items: BTreeMap::new(),
            found_top_level_types: HashMap::new(),
            found_top_level_comments: HashMap::new(),
            found_top_level_exports: Vec::new(),
        }
    }

    /**
        Parses the given Luau type definitions into parser items.

        The parser items will be stored internally and can be converted
        into usable definition items using [`DefinitionsParser::drain`].
    */
    pub fn parse<S>(&mut self, contents: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        // Parse contents into top-level parser items for later use
        let mut found_top_level_items = BTreeMap::new();
        let mut found_top_level_types = HashMap::new();
        let mut found_top_level_comments = HashMap::new();
        let mut found_top_level_exports = HashSet::new();
        let ast =
            full_moon::parse(contents.as_ref()).context("Failed to parse type definitions")?;
        for stmt in ast.nodes().stmts() {
            if let Some((exported, declaration, token_reference)) = match stmt {
                Stmt::ExportedTypeDeclaration(exp) => {
                    Some((true, exp.type_declaration(), exp.export_token()))
                }
                Stmt::TypeDeclaration(typ) => Some((false, typ, typ.type_token())),
                _ => None,
            } {
                let name = declaration.type_name().token().to_string();
                let comment = find_token_moonwave_comment(token_reference);
                found_top_level_items.insert(
                    name.clone(),
                    DefinitionsParserItem {
                        name: name.clone(),
                        comment: comment.clone(),
                        type_info: declaration.type_definition().clone(),
                    },
                );
                found_top_level_types.insert(name.clone(), declaration.type_definition().clone());
                found_top_level_comments.insert(name.clone(), comment);
                if exported {
                    found_top_level_exports.insert(name);
                }
            }
        }
        // Store results
        self.found_top_level_items = found_top_level_items;
        self.found_top_level_types = found_top_level_types;
        self.found_top_level_comments = found_top_level_comments;
        self.found_top_level_exports = found_top_level_exports.into_iter().collect();
        Ok(())
    }

    fn convert_parser_item_into_doc_item(
        &self,
        item: DefinitionsParserItem,
        kind: Option<DefinitionsItemKind>,
    ) -> DefinitionsItem {
        let mut builder = DefinitionsItemBuilder::new()
            .with_kind(kind.unwrap_or_else(|| item.type_info.parse_definitions_kind()))
            .with_name(&item.name)
            .with_type(item.type_info.to_string());
        if self.found_top_level_exports.contains(&item.name) {
            builder = builder.as_exported();
        }
        if let Some(comment) = item.comment {
            builder = builder.with_children(&parse_moonwave_style_comment(&comment));
        }
        if let Some(args) = item
            .type_info
            .extract_args_normalized(&self.found_top_level_types)
        {
            builder = builder.with_args(&args);
        }
        if let TypeInfo::Table { fields, .. } = item.type_info {
            for field in fields.iter() {
                if let TypeFieldKey::Name(name) = field.key() {
                    builder = builder.with_child(self.convert_parser_item_into_doc_item(
                        DefinitionsParserItem {
                            name: name.token().to_string(),
                            comment: find_token_moonwave_comment(name),
                            type_info: field.value().clone(),
                        },
                        None,
                    ));
                }
            }
        }
        builder.build().unwrap()
    }

    /**
        Converts currently stored parser items into definition items.

        This will consume (drain) all stored parser items, leaving the parser empty.
    */
    #[allow(clippy::unnecessary_wraps)]
    pub fn drain(&mut self) -> Result<Vec<DefinitionsItem>> {
        let mut resulting_items = Vec::new();
        for top_level_item in self.found_top_level_items.values() {
            resulting_items
                .push(self.convert_parser_item_into_doc_item(top_level_item.clone(), None));
        }
        self.found_top_level_items = BTreeMap::new();
        self.found_top_level_types = HashMap::new();
        self.found_top_level_comments = HashMap::new();
        self.found_top_level_exports = Vec::new();
        Ok(resulting_items)
    }
}

fn find_token_moonwave_comment(token: &TokenReference) -> Option<String> {
    token
        .leading_trivia()
        .filter_map(|trivia| match trivia.token_type() {
            TokenType::MultiLineComment { blocks, comment } if blocks == &1 => Some(comment),
            _ => None,
        })
        .last()
        .map(ToString::to_string)
}
