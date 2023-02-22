use anyhow::{Context, Result};
use full_moon::{
    ast::{
        types::{TypeArgument, TypeFieldKey, TypeInfo},
        Stmt,
    },
    tokenizer::{TokenReference, TokenType},
};
use regex::Regex;

use super::{
    builder::DefinitionsItemBuilder, item::DefinitionsItem, kind::DefinitionsItemKind,
    moonwave::parse_moonwave_style_comment,
};

pub const PIPE_SEPARATOR: &str = " | ";

#[derive(Debug, Clone)]
struct DefinitionsParserItem {
    name: String,
    comment: Option<String>,
    type_info: TypeInfo,
}

impl DefinitionsParserItem {
    fn into_doc_item(self, type_definition_declares: &Vec<String>) -> DefinitionsItem {
        let mut builder = DefinitionsItemBuilder::new()
            .with_kind(DefinitionsItemKind::from(&self.type_info))
            .with_name(&self.name);
        if type_definition_declares.contains(&self.name) {
            builder = builder.as_exported();
        }
        if let Some(comment) = self.comment {
            builder = builder.with_children(&parse_moonwave_style_comment(&comment));
        }
        if let Some(args) = try_extract_normalized_function_args(&self.type_info) {
            builder = builder.with_arg_types(&args);
        }
        if let TypeInfo::Table { fields, .. } = self.type_info {
            for field in fields.iter() {
                if let TypeFieldKey::Name(name) = field.key() {
                    builder = builder.with_child(
                        Self {
                            name: name.token().to_string(),
                            comment: find_token_moonwave_comment(name),
                            type_info: field.value().clone(),
                        }
                        .into_doc_item(type_definition_declares),
                    );
                }
            }
        }
        builder.build().unwrap()
    }
}

impl From<&TypeInfo> for DefinitionsItemKind {
    fn from(value: &TypeInfo) -> Self {
        match value {
            TypeInfo::Array { .. } | TypeInfo::Table { .. } => DefinitionsItemKind::Table,
            TypeInfo::Basic(_) | TypeInfo::String(_) => DefinitionsItemKind::Property,
            TypeInfo::Optional { base, .. } => Self::from(base.as_ref()),
            TypeInfo::Tuple { types, .. } => {
                let mut kinds = types.iter().map(Self::from).collect::<Vec<_>>();
                let kinds_all_the_same = kinds.windows(2).all(|w| w[0] == w[1]);
                if kinds_all_the_same && !kinds.is_empty() {
                    kinds.pop().unwrap()
                } else {
                    unimplemented!(
                        "Missing support for tuple with differing types in type definitions parser",
                    )
                }
            }
            TypeInfo::Union { left, right, .. } | TypeInfo::Intersection { left, right, .. } => {
                let kind_left = Self::from(left.as_ref());
                let kind_right = Self::from(right.as_ref());
                if kind_left == kind_right {
                    kind_left
                } else {
                    unimplemented!(
                        "Missing support for union/intersection with differing types in type definitions parser",
                    )
                }
            }
            typ if type_info_is_fn(typ) => DefinitionsItemKind::Function,
            typ => unimplemented!(
                "Missing support for TypeInfo in type definitions parser:\n{}",
                typ.to_string()
            ),
        }
    }
}

fn parse_type_definitions_declares(contents: &str) -> (String, Vec<String>) {
    // TODO: Properly handle the "declare class" syntax, for now we just skip it
    let mut no_class_declares = contents.to_string();
    while let Some(dec) = no_class_declares.find("\ndeclare class") {
        let end = no_class_declares.find("\nend").unwrap();
        let before = &no_class_declares[0..dec];
        let after = &no_class_declares[end + 4..];
        no_class_declares = format!("{before}{after}");
    }
    let regex_declare = Regex::new(r#"declare (\w+): "#).unwrap();
    let resulting_contents = regex_declare
        .replace_all(&no_class_declares, "export type $1 =")
        .to_string();
    let found_declares = regex_declare
        .captures_iter(&no_class_declares)
        .map(|cap| cap[1].to_string())
        .collect();
    (resulting_contents, found_declares)
}

pub fn parse_type_definitions_into_doc_items<S>(contents: S) -> Result<Vec<DefinitionsItem>>
where
    S: AsRef<str>,
{
    let mut found_top_level_items = Vec::new();
    let (type_definition_contents, type_definition_declares) =
        parse_type_definitions_declares(contents.as_ref());
    let ast =
        full_moon::parse(&type_definition_contents).context("Failed to parse type definitions")?;
    for stmt in ast.nodes().stmts() {
        if let Some((declaration, token_reference)) = match stmt {
            Stmt::ExportedTypeDeclaration(exp) => {
                Some((exp.type_declaration(), exp.export_token()))
            }
            Stmt::TypeDeclaration(typ) => Some((typ, typ.type_token())),
            _ => None,
        } {
            found_top_level_items.push(DefinitionsParserItem {
                name: declaration.type_name().token().to_string(),
                comment: find_token_moonwave_comment(token_reference),
                type_info: declaration.type_definition().clone(),
            });
        }
    }
    Ok(found_top_level_items
        .drain(..)
        .map(|visitor_item| visitor_item.into_doc_item(&type_definition_declares))
        .collect())
}

fn simple_stringify_type_info(typ: &TypeInfo, parent_typ: Option<&TypeInfo>) -> String {
    match typ {
        TypeInfo::Array { type_info, .. } => {
            format!("{{ {} }}", simple_stringify_type_info(type_info, Some(typ)))
        }
        TypeInfo::Basic(tok) => match parent_typ {
            Some(TypeInfo::Callback { generics, .. }) => {
                if let Some(generics) = generics {
                    // If the function that contains this arg has generic and a
                    // generic is the same as this token, we stringify it as any
                    if generics
                        .generics()
                        .iter()
                        .any(|g| g.to_string() == tok.token().to_string())
                    {
                        "any".to_string()
                    } else {
                        tok.token().to_string()
                    }
                } else {
                    tok.token().to_string()
                }
            }
            _ => tok.token().to_string(),
        },
        TypeInfo::String(str) => str.token().to_string(),
        TypeInfo::Boolean(_) => "boolean".to_string(),
        TypeInfo::Callback { .. } => "function".to_string(),
        TypeInfo::Optional { base, .. } => {
            format!("{}?", simple_stringify_type_info(base, Some(typ)))
        }
        TypeInfo::Table { .. } => "table".to_string(),
        TypeInfo::Union { left, right, .. } => {
            format!(
                "{}{PIPE_SEPARATOR}{}",
                simple_stringify_type_info(left, Some(typ)),
                simple_stringify_type_info(right, Some(typ))
            )
        }
        // TODO: Stringify custom table types properly, these show up as basic tokens
        // and we should be able to look up the real type using found top level items
        _ => "...".to_string(),
    }
}

fn type_info_is_fn(typ: &TypeInfo) -> bool {
    match typ {
        TypeInfo::Callback { .. } => true,
        TypeInfo::Tuple { types, .. } => types.iter().all(type_info_is_fn),
        TypeInfo::Union { left, right, .. } | TypeInfo::Intersection { left, right, .. } => {
            type_info_is_fn(left) || type_info_is_fn(right)
        }
        _ => false,
    }
}

fn type_info_extract_args<'a>(
    typ: &'a TypeInfo,
    base: Vec<Vec<&'a TypeArgument>>,
) -> Vec<Vec<&'a TypeArgument>> {
    match typ {
        TypeInfo::Callback { arguments, .. } => {
            let mut result = base.clone();
            result.push(arguments.iter().collect::<Vec<_>>());
            result
        }
        TypeInfo::Tuple { types, .. } => type_info_extract_args(
            types.iter().next().expect("Function tuple type was empty"),
            base.clone(),
        ),
        TypeInfo::Union { left, right, .. } | TypeInfo::Intersection { left, right, .. } => {
            let mut result = base.clone();
            result = type_info_extract_args(left, result.clone());
            result = type_info_extract_args(right, result.clone());
            result
        }
        _ => base,
    }
}

fn try_extract_normalized_function_args(typ: &TypeInfo) -> Option<Vec<String>> {
    if type_info_is_fn(typ) {
        let mut type_args_multi = type_info_extract_args(typ, Vec::new());
        match type_args_multi.len() {
            0 => None,
            1 => Some(
                // We got a normal function with some known list of args, and we will
                // stringify the arg types into simple ones such as "function", "table", ..
                type_args_multi
                    .pop()
                    .unwrap()
                    .iter()
                    .map(|type_arg| simple_stringify_type_info(type_arg.type_info(), Some(typ)))
                    .collect(),
            ),
            _ => {
                // We got a union or intersection function, meaning it has
                // several different overloads that accept different args
                let mut unified_args = Vec::new();
                for index in 0..type_args_multi
                    .iter()
                    .fold(0, |acc, type_args| acc.max(type_args.len()))
                {
                    // Gather function arg type strings for all
                    // of the different variants of this function
                    let mut type_arg_strings = type_args_multi
                        .iter()
                        .filter_map(|type_args| type_args.get(index))
                        .map(|type_arg| simple_stringify_type_info(type_arg.type_info(), Some(typ)))
                        .collect::<Vec<_>>();
                    if type_arg_strings.len() < type_args_multi.len() {
                        for _ in type_arg_strings.len()..type_args_multi.len() {
                            type_arg_strings.push("nil".to_string());
                        }
                    }
                    // Type arg strings may themselves be stringified to something like number | string so we
                    // will split that out to be able to handle it better with the following unification process
                    let mut type_arg_strings_sep = Vec::new();
                    for type_arg_string in type_arg_strings.drain(..) {
                        for typ_arg_string_inner in type_arg_string.split(PIPE_SEPARATOR) {
                            type_arg_strings_sep.push(typ_arg_string_inner.to_string());
                        }
                    }
                    // Find out if we have any nillable type, to know if we
                    // should make the entire arg type union nillable or not
                    let has_any_optional = type_arg_strings_sep
                        .iter()
                        .any(|s| s == "nil" || s.ends_with('?'));
                    // Filter out any nils or optional markers (?),
                    // we will add this back at the end if necessary
                    let mut type_arg_strings_non_nil = type_arg_strings_sep
                        .iter()
                        .filter(|s| *s != "nil")
                        .map(|s| s.trim_end_matches('?').to_string())
                        .collect::<Vec<_>>();
                    type_arg_strings_non_nil.sort(); // Need to sort for dedup
                    type_arg_strings_non_nil.dedup(); // Dedup to get rid of redundant types such as string | string
                    unified_args.push(if has_any_optional {
                        if type_arg_strings_non_nil.len() == 1 {
                            format!("{}?", type_arg_strings_non_nil.pop().unwrap())
                        } else {
                            format!("({})?", type_arg_strings_non_nil.join(PIPE_SEPARATOR))
                        }
                    } else {
                        type_arg_strings_non_nil.join(PIPE_SEPARATOR)
                    });
                }
                Some(unified_args)
            }
        }
    } else {
        None
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
        .map(|comment| comment.trim().to_string())
}
