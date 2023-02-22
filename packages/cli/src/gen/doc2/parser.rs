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
    builder::DocItemBuilder, item::DocItem, kind::DocItemKind,
    moonwave::parse_moonwave_style_comment,
};

pub const PIPE_SEPARATOR: &str = " | ";

#[derive(Debug, Clone)]
struct DocVisitorItem {
    name: String,
    comment: Option<String>,
    type_info: TypeInfo,
}

impl From<DocVisitorItem> for DocItem {
    fn from(value: DocVisitorItem) -> Self {
        let mut builder = DocItemBuilder::new()
            .with_kind(DocItemKind::from(&value.type_info))
            .with_name(&value.name);
        if let Some(comment) = value.comment {
            builder = builder.with_children(&parse_moonwave_style_comment(&comment));
        }
        if let Some(args) = try_extract_normalized_function_args(&value.type_info) {
            println!("{} > {args:?}", value.name);
            builder = builder.with_arg_types(&args);
        }
        if let TypeInfo::Table { fields, .. } = value.type_info {
            for field in fields.iter() {
                if let TypeFieldKey::Name(name) = field.key() {
                    builder = builder.with_child(DocItem::from(DocVisitorItem {
                        name: name.token().to_string(),
                        comment: find_token_moonwave_comment(name),
                        type_info: field.value().clone(),
                    }));
                }
            }
        }
        builder.build().unwrap()
    }
}

impl From<&TypeInfo> for DocItemKind {
    fn from(value: &TypeInfo) -> Self {
        match value {
            TypeInfo::Array { .. } | TypeInfo::Table { .. } => DocItemKind::Table,
            TypeInfo::Basic(_) | TypeInfo::String(_) => DocItemKind::Property,
            TypeInfo::Optional { base, .. } => DocItemKind::from(base.as_ref()),
            TypeInfo::Tuple { types, .. } => {
                let mut kinds = types.iter().map(DocItemKind::from).collect::<Vec<_>>();
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
                let kind_left = DocItemKind::from(left.as_ref());
                let kind_right = DocItemKind::from(right.as_ref());
                if kind_left == kind_right {
                    kind_left
                } else {
                    unimplemented!(
                        "Missing support for union/intersection with differing types in type definitions parser",
                    )
                }
            }
            typ if type_info_is_fn(typ) => DocItemKind::Function,
            typ => unimplemented!(
                "Missing support for TypeInfo in type definitions parser:\n{}",
                typ.to_string()
            ),
        }
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
                type_info: declaration.type_definition().clone(),
            });
        }
    }
    Ok(found_top_level_items.drain(..).map(DocItem::from).collect())
}

fn simple_stringify_type_info(typ: &TypeInfo) -> String {
    match typ {
        TypeInfo::Array { type_info, .. } => {
            format!("{{ {} }}", simple_stringify_type_info(type_info))
        }
        TypeInfo::Basic(tok) => {
            if tok.token().to_string() == "T" {
                "any".to_string() // HACK: Assume that any literal type "T" is a generic that accepts any value
            } else {
                tok.token().to_string()
            }
        }
        TypeInfo::String(str) => str.token().to_string(),
        TypeInfo::Boolean(_) => "boolean".to_string(),
        TypeInfo::Callback { .. } => "function".to_string(),
        TypeInfo::Optional { base, .. } => format!("{}?", simple_stringify_type_info(base)),
        TypeInfo::Table { .. } => "table".to_string(),
        TypeInfo::Union { left, right, .. } => {
            format!(
                "{}{PIPE_SEPARATOR}{}",
                simple_stringify_type_info(left),
                simple_stringify_type_info(right)
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
                    .map(|type_arg| simple_stringify_type_info(type_arg.type_info()))
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
                        .map(|type_arg| simple_stringify_type_info(type_arg.type_info()))
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
