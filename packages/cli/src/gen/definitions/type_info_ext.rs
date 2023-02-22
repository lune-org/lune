use full_moon::ast::types::{TypeArgument, TypeInfo};

use super::kind::DefinitionsItemKind;

pub const PIPE_SEPARATOR: &str = " | ";

pub(super) trait TypeInfoExt {
    fn is_fn(&self) -> bool;
    fn to_definitions_kind(&self) -> DefinitionsItemKind;
    fn stringify_simple(&self, parent_typ: Option<&TypeInfo>) -> String;
    fn extract_args<'a>(&'a self, base: Vec<Vec<&'a TypeArgument>>) -> Vec<Vec<&'a TypeArgument>>;
    fn extract_args_normalized(&self) -> Option<Vec<String>>;
}

impl TypeInfoExt for TypeInfo {
    fn is_fn(&self) -> bool {
        match self {
            TypeInfo::Callback { .. } => true,
            TypeInfo::Tuple { types, .. } => types.iter().all(Self::is_fn),
            TypeInfo::Union { left, right, .. } | TypeInfo::Intersection { left, right, .. } => {
                left.is_fn() || right.is_fn()
            }
            _ => false,
        }
    }

    fn to_definitions_kind(&self) -> DefinitionsItemKind {
        match self {
            TypeInfo::Array { .. } | TypeInfo::Table { .. } => DefinitionsItemKind::Table,
            TypeInfo::Basic(_) | TypeInfo::String(_) => DefinitionsItemKind::Property,
            TypeInfo::Optional { base, .. } => Self::to_definitions_kind(base.as_ref()),
            TypeInfo::Tuple { types, .. } => {
                let mut kinds = types
                    .iter()
                    .map(Self::to_definitions_kind)
                    .collect::<Vec<_>>();
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
                let kind_left = Self::to_definitions_kind(left.as_ref());
                let kind_right = Self::to_definitions_kind(right.as_ref());
                if kind_left == kind_right {
                    kind_left
                } else {
                    unimplemented!(
                        "Missing support for union/intersection with differing types in type definitions parser",
                    )
                }
            }
            typ if typ.is_fn() => DefinitionsItemKind::Function,
            typ => unimplemented!(
                "Missing support for TypeInfo in type definitions parser:\n{}",
                typ.to_string()
            ),
        }
    }

    fn stringify_simple(&self, parent_typ: Option<&TypeInfo>) -> String {
        match self {
            TypeInfo::Array { type_info, .. } => {
                format!("{{ {} }}", type_info.as_ref().stringify_simple(Some(self)))
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
                format!("{}?", base.as_ref().stringify_simple(Some(self)))
            }
            TypeInfo::Table { .. } => "table".to_string(),
            TypeInfo::Union { left, right, .. } => {
                format!(
                    "{}{PIPE_SEPARATOR}{}",
                    left.as_ref().stringify_simple(Some(self)),
                    right.as_ref().stringify_simple(Some(self))
                )
            }
            // TODO: Stringify custom table types properly, these show up as basic tokens
            // and we should be able to look up the real type using found top level items
            _ => "...".to_string(),
        }
    }

    fn extract_args<'a>(&'a self, base: Vec<Vec<&'a TypeArgument>>) -> Vec<Vec<&'a TypeArgument>> {
        match self {
            TypeInfo::Callback { arguments, .. } => {
                let mut result = base.clone();
                result.push(arguments.iter().collect::<Vec<_>>());
                result
            }
            TypeInfo::Tuple { types, .. } => types
                .iter()
                .next()
                .expect("Function tuple type was empty")
                .extract_args(base.clone()),
            TypeInfo::Union { left, right, .. } | TypeInfo::Intersection { left, right, .. } => {
                let mut result = base.clone();
                result = left.extract_args(result.clone());
                result = right.extract_args(result.clone());
                result
            }
            _ => base,
        }
    }

    fn extract_args_normalized(&self) -> Option<Vec<String>> {
        if self.is_fn() {
            let mut type_args_multi = self.extract_args(Vec::new());
            match type_args_multi.len() {
                0 => None,
                1 => Some(
                    // We got a normal function with some known list of args, and we will
                    // stringify the arg types into simple ones such as "function", "table", ..
                    type_args_multi
                        .pop()
                        .unwrap()
                        .iter()
                        .map(|type_arg| type_arg.type_info().stringify_simple(Some(self)))
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
                            .map(|type_arg| type_arg.type_info().stringify_simple(Some(self)))
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
}
