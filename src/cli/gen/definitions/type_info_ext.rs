use std::collections::HashMap;

use full_moon::{
    ast::types::{TypeArgument, TypeInfo},
    tokenizer::{Symbol, Token, TokenReference, TokenType},
    ShortString,
};

use super::{
    item::{DefinitionsItemFunctionArg, DefinitionsItemFunctionRet},
    kind::DefinitionsItemKind,
};

pub(crate) trait TypeInfoExt {
    fn is_fn(&self) -> bool;
    fn parse_definitions_kind(&self) -> DefinitionsItemKind;
    fn stringify_simple(
        &self,
        parent_typ: Option<&TypeInfo>,
        type_lookup_table: &HashMap<String, TypeInfo>,
    ) -> String;
    fn extract_args(&self) -> Vec<TypeArgument>;
    fn extract_args_normalized(
        &self,
        type_lookup_table: &HashMap<String, TypeInfo>,
    ) -> Option<Vec<DefinitionsItemFunctionArg>>;
    // fn extract_rets(&self) -> Vec<TypeArgument>;
    // fn extract_rets_normalized(
    //     &self,
    //     type_lookup_table: &HashMap<String, TypeInfo>,
    // ) -> Option<Vec<DefinitionsItemFunctionRet>>;
}

impl TypeInfoExt for TypeInfo {
    /**
        Checks if this type represents a function or not.

        If the type is a tuple, union, or intersection, it will be checked recursively.
    */
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

    /**
        Parses the definitions item kind from the type.

        If the type is a tupe, union, or intersection, all the inner types
        are required to be equivalent in terms of definitions item kinds.
    */
    fn parse_definitions_kind(&self) -> DefinitionsItemKind {
        match self {
            TypeInfo::Array { .. } | TypeInfo::Table { .. } => DefinitionsItemKind::Table,
            TypeInfo::Basic(_) | TypeInfo::String(_) => DefinitionsItemKind::Property,
            TypeInfo::Optional { base, .. } => Self::parse_definitions_kind(base.as_ref()),
            TypeInfo::Tuple { types, .. } => {
                let mut kinds = types
                    .iter()
                    .map(Self::parse_definitions_kind)
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
                let kind_left = Self::parse_definitions_kind(left.as_ref());
                let kind_right = Self::parse_definitions_kind(right.as_ref());
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

    /**
        Stringifies the type into a simplified type string.

        The simplified type string match one of the following formats:

        * `any`
        * `boolean`
        * `string`
        * `function`
        * `table`
        * `CustomTypeName`
        * `TypeName?`
        * `TypeName | OtherTypeName`
        * `{ TypeName }`
        * `"string-literal"`
    */
    fn stringify_simple(
        &self,
        parent_typ: Option<&TypeInfo>,
        type_lookup_table: &HashMap<String, TypeInfo>,
    ) -> String {
        match self {
            TypeInfo::Array { type_info, .. } => {
                format!(
                    "{{ {} }}",
                    type_info
                        .as_ref()
                        .stringify_simple(Some(self), type_lookup_table)
                )
            }
            TypeInfo::Basic(tok) => {
                let tok_str = tok.token().to_string().trim().to_string();
                let mut any_str = None;
                // If the function that contains this arg has generic and a
                // generic is the same as this token, we stringify it as any
                if let Some(parent) = parent_typ {
                    if let Some(TypeInfo::Callback {
                        generics: Some(callback_generics),
                        ..
                    }) = try_extract_callback_type_info(parent)
                    {
                        if callback_generics
                            .generics()
                            .iter()
                            .any(|g| g.to_string() == tok_str)
                        {
                            any_str = Some("any".to_string());
                        }
                    }
                }
                // Also check if we got a referenced type, meaning that it
                // exists in the lookup table of global types passed to us
                if let Some(any_str) = any_str {
                    any_str
                } else if let Some(referenced_typ) = type_lookup_table.get(&tok_str) {
                    referenced_typ.stringify_simple(None, type_lookup_table)
                } else {
                    tok_str
                }
            }
            TypeInfo::String(str) => str.token().to_string(),
            TypeInfo::Boolean(_) => "boolean".to_string(),
            TypeInfo::Callback { .. } => "function".to_string(),
            TypeInfo::Optional { base, .. } => {
                format!(
                    "{}?",
                    base.as_ref()
                        .stringify_simple(Some(self), type_lookup_table)
                )
            }
            TypeInfo::Table { .. } => "table".to_string(),
            TypeInfo::Union { left, right, .. } => {
                format!(
                    "{} {} {}",
                    left.as_ref()
                        .stringify_simple(Some(self), type_lookup_table),
                    Symbol::Pipe,
                    right
                        .as_ref()
                        .stringify_simple(Some(self), type_lookup_table)
                )
            }
            // FUTURE: Is there any other type that we can
            // stringify to a primitive in an obvious way?
            _ => "...".to_string(),
        }
    }

    fn extract_args(&self) -> Vec<TypeArgument> {
        if self.is_fn() {
            match self {
                TypeInfo::Callback { arguments, .. } => {
                    arguments.iter().cloned().collect::<Vec<_>>()
                }
                TypeInfo::Tuple { types, .. } => types
                    .iter()
                    .next()
                    .expect("Function tuple type was empty")
                    .extract_args(),
                TypeInfo::Union { left, right, .. }
                | TypeInfo::Intersection { left, right, .. } => {
                    let mut result = Vec::new();
                    result = merge_type_argument_vecs(result, left.extract_args());
                    result = merge_type_argument_vecs(result, right.extract_args());
                    result
                }
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    fn extract_args_normalized(
        &self,
        type_lookup_table: &HashMap<String, TypeInfo>,
    ) -> Option<Vec<DefinitionsItemFunctionArg>> {
        if self.is_fn() {
            let args_stringified_not_normalized = self
                .extract_args()
                .iter()
                .map(|type_arg| {
                    (
                        type_arg
                            .name()
                            .map_or_else(|| "_".to_string(), |n| n.0.to_string()),
                        type_arg.type_info().to_string(),
                        type_arg
                            .type_info()
                            .stringify_simple(Some(self), type_lookup_table),
                    )
                })
                .collect::<Vec<_>>();
            let mut args = Vec::new();
            for (arg_name, arg_typedef, arg_typedef_simplified) in args_stringified_not_normalized {
                args.push(DefinitionsItemFunctionArg::new(
                    arg_name,
                    arg_typedef,
                    normalize_type(&arg_typedef_simplified),
                ));
            }
            Some(args)
        } else {
            None
        }
    }
}

fn try_extract_callback_type_info(type_info: &TypeInfo) -> Option<&TypeInfo> {
    match type_info {
        TypeInfo::Callback { .. } => Some(type_info),
        TypeInfo::Tuple { types, .. } => types.iter().find_map(try_extract_callback_type_info),
        TypeInfo::Union { left, right, .. } | TypeInfo::Intersection { left, right, .. } => {
            try_extract_callback_type_info(left).or_else(|| try_extract_callback_type_info(right))
        }
        _ => None,
    }
}

fn make_empty_type_argument() -> TypeArgument {
    TypeArgument::new(TypeInfo::Basic(TokenReference::new(
        vec![],
        Token::new(TokenType::Symbol {
            symbol: Symbol::Nil,
        }),
        vec![],
    )))
}

fn merge_type_arguments(left: TypeArgument, right: TypeArgument) -> TypeArgument {
    TypeArgument::new(TypeInfo::Union {
        left: Box::new(left.type_info().clone()),
        pipe: TokenReference::new(
            vec![Token::new(TokenType::Whitespace {
                characters: ShortString::new(" "),
            })],
            Token::new(TokenType::Symbol {
                symbol: Symbol::Pipe,
            }),
            vec![Token::new(TokenType::Whitespace {
                characters: ShortString::new(" "),
            })],
        ),
        right: Box::new(right.type_info().clone()),
    })
}

fn merge_type_argument_vecs(
    existing: Vec<TypeArgument>,
    new: Vec<TypeArgument>,
) -> Vec<TypeArgument> {
    let mut result = Vec::new();
    for (index, argument) in new.iter().enumerate() {
        if let Some(existing) = existing.get(index) {
            result.push(merge_type_arguments(existing.clone(), argument.clone()));
        } else {
            result.push(merge_type_arguments(
                make_empty_type_argument(),
                argument.clone(),
            ));
        }
    }
    result
}

fn normalize_type(simplified: &str) -> String {
    let separator = format!(" {} ", Symbol::Pipe);
    let arg_parts = simplified.split(&separator).collect::<Vec<_>>();
    // Check if we got any optional arg, if so then the entire possible
    // union of args will be optional when merged together / normalized
    let is_optional = arg_parts
        .iter()
        .any(|part| part == &"nil" || part.ends_with('?'));
    // Get rid of any nils or optional markers since we keep track of it above
    let mut arg_parts_no_nils = arg_parts
        .iter()
        .filter_map(|arg_part| {
            if arg_part == &"nil" {
                None
            } else {
                Some(arg_part.trim_end_matches('?'))
            }
        })
        .collect::<Vec<_>>();
    arg_parts_no_nils.sort_unstable(); // Sort the args to be able to dedup
    arg_parts_no_nils.dedup(); // Deduplicate types that are the exact same shape
    if is_optional {
        if arg_parts_no_nils.len() > 1 {
            // A union of args that is nillable should be enclosed in parens to make
            // it more clear that the entire arg is nillable and not just the last type
            format!("({})?", arg_parts_no_nils.join(&separator))
        } else {
            // Just one nillable arg, does not need any parens
            format!("{}?", arg_parts_no_nils.first().unwrap())
        }
    } else {
        arg_parts_no_nils.join(&separator)
    }
}
