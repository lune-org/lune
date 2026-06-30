/*!
    Matcher for the `QueryDescendants` selector grammar.

    A complex selector is matched against a candidate instance right-to-left:
    the rightmost compound (the "subject") must match the candidate itself, then
    the remaining compounds are matched against the candidate's ancestor chain,
    up to but never past the current scope (the query root, or a `:has` subject).
*/

use rbx_dom_weak::types::Variant as DomValue;

use crate::instance::Instance;
use crate::shared::instance::find_property_info;

use super::QueryError;
use super::ast::{Combinator, ComplexPart, ComplexSelector, CompoundSelector, SimpleSelector};

/**
    Compares a stored [`DomValue`] against a selector value literal.

    A selector value is always a raw string (quotes only widen the allowed
    character set); the comparison coerces it to the stored value's runtime
    type, matching Roblox: a numeric value parses the literal as a number, a
    boolean accepts only `true`/`false`, strings compare exactly
    (case-sensitive).

    Returns `Err(type_name)` if the stored value is of a type that cannot be
    compared (e.g. `Color3`, `Vector3`), so the caller can raise the appropriate
    "not supported for comparison" error.
*/
#[allow(clippy::cast_precision_loss, clippy::float_cmp)]
fn compare_value(stored: &DomValue, literal: &str) -> Result<bool, String> {
    // NOTE: Numeric comparison is intentionally exact (not within an epsilon).
    // Roblox compares the parsed literal against the stored value exactly,
    // meaning `[X = 4]` must not match a value of `4.0001`.
    Ok(match stored {
        DomValue::Bool(b) => match literal {
            "true" => *b,
            "false" => !*b,
            _ => false,
        },
        DomValue::Int32(n) => literal.parse::<f64>().is_ok_and(|v| f64::from(*n) == v),
        DomValue::Int64(n) => literal.parse::<f64>().is_ok_and(|v| *n as f64 == v),
        DomValue::Float32(n) => literal.parse::<f64>().is_ok_and(|v| f64::from(*n) == v),
        DomValue::Float64(n) => literal.parse::<f64>().is_ok_and(|v| *n == v),
        DomValue::String(s) => s == literal,
        DomValue::BinaryString(s) => AsRef::<[u8]>::as_ref(s) == literal.as_bytes(),
        DomValue::ContentId(s) => AsRef::<str>::as_ref(s) == literal,
        other => return Err(format!("{:?}", other.ty())),
    })
}

/**
    Resolves whether an enum value (by its numeric representation) corresponds to
    the given enum item name, using the reflection database.
*/
fn enum_value_matches(enum_name: &str, value: u32, literal: &str) -> bool {
    let db = rbx_reflection_database::get().unwrap();
    db.enums.get(enum_name).is_some_and(|desc| {
        desc.items
            .iter()
            .any(|(name, item_value)| name.as_ref() == literal && *item_value == value)
    })
}

fn match_property(node: Instance, name: &str, literal: &str) -> Result<bool, QueryError> {
    let Some(info) = find_property_info(node.get_class_name(), name) else {
        // The property does not exist on this class - no match, no error.
        return Ok(false);
    };

    // Enum properties are compared against the enum item name (Roblox supports
    // these even though they are not a plain primitive).
    if let Some(enum_name) = &info.enum_name {
        let effective = match node.get_property(name) {
            Some(DomValue::Enum(enum_value)) => Some(enum_value.to_u32()),
            _ => info.enum_default,
        };
        return Ok(effective.is_some_and(|value| enum_value_matches(enum_name, value, literal)));
    }

    // Otherwise use the effective value (the explicitly stored value, falling
    // back to the class default - Roblox compares against the live/effective
    // value, so e.g. `[CanCollide = true]` matches an unmodified Part).
    let effective = node
        .get_property(name)
        .or_else(|| info.value_default.cloned());
    let Some(stored) = effective else {
        return Ok(false);
    };

    compare_value(&stored, literal).map_err(|ty| QueryError::UnsupportedPropertyType {
        name: name.to_string(),
        ty,
    })
}

fn match_attribute(node: Instance, name: &str, literal: &str) -> Result<bool, QueryError> {
    let Some(stored) = node.get_attribute(name) else {
        return Ok(false);
    };
    compare_value(&stored, literal).map_err(|ty| QueryError::UnsupportedAttributeType {
        name: name.to_string(),
        ty,
    })
}

fn matches_simple(
    node: Instance,
    simple: &SimpleSelector,
    scope: Instance,
) -> Result<bool, QueryError> {
    match simple {
        SimpleSelector::Type(class_name) => Ok(node.is_a(class_name)),
        SimpleSelector::Tag(tag) => Ok(node.has_tag(tag)),
        SimpleSelector::Name(name) => Ok(node.get_name() == *name),
        SimpleSelector::Property { name, value } => match_property(node, name, value),
        SimpleSelector::AttributeExists { name } => Ok(node.get_attribute(name).is_some()),
        SimpleSelector::Attribute { name, value } => match_attribute(node, name, value),
        SimpleSelector::Not(list) => {
            // Matches iff the node matches NONE of the inner selectors, in the
            // current scope.
            for complex in &list.0 {
                if matches_complex(node, complex, scope)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        SimpleSelector::Has(list) => {
            // Matches iff some descendant of the node matches one of the inner
            // selectors, evaluated relative to the node (the new scope).
            for candidate in node.get_descendants_preorder() {
                for complex in &list.0 {
                    if matches_complex(candidate, complex, node)? {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        }
    }
}

fn matches_compound(
    node: Instance,
    compound: &CompoundSelector,
    scope: Instance,
) -> Result<bool, QueryError> {
    for simple in &compound.0 {
        if !matches_simple(node, simple, scope)? {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn matches_complex(
    node: Instance,
    complex: &ComplexSelector,
    scope: Instance,
) -> Result<bool, QueryError> {
    let parts = &complex.parts;
    let last = parts.len() - 1;

    // The rightmost compound (the "subject") must match the candidate itself.
    if !matches_compound(node, &parts[last].compound, scope)? {
        return Ok(false);
    }

    match_left(node, parts, last, complex.leading, scope)
}

/**
    Walks the ancestor chain leftward, matching `parts[idx]` (already matched at
    `node`) back to `parts[0]`, then enforces the leading combinator.
*/
fn match_left(
    node: Instance,
    parts: &[ComplexPart],
    idx: usize,
    leading: Combinator,
    scope: Instance,
) -> Result<bool, QueryError> {
    if idx == 0 {
        return Ok(enforce_leading(node, leading, scope));
    }

    let combinator = parts[idx].combinator;
    let target = &parts[idx - 1].compound;

    match combinator {
        Combinator::Child => {
            // The parent must be a strict descendant of the scope (the scope
            // itself is never an eligible intermediate match).
            if let Some(parent) = node.get_parent()
                && parent != scope
                && matches_compound(parent, target, scope)?
            {
                return match_left(parent, parts, idx - 1, leading, scope);
            }
            Ok(false)
        }
        Combinator::Descendant => {
            // Try every ancestor up to (but not including) the scope, with
            // backtracking - a nearer ancestor matching `target` may fail
            // further left while a farther one succeeds.
            let mut ancestor = node.get_parent();
            while let Some(current) = ancestor {
                if current == scope {
                    break;
                }
                if matches_compound(current, target, scope)?
                    && match_left(current, parts, idx - 1, leading, scope)?
                {
                    return Ok(true);
                }
                ancestor = current.get_parent();
            }
            Ok(false)
        }
    }
}

/**
    Enforces the leading combinator of a complex selector against the scope.
    The node here is whatever matched the leftmost compound.
*/
fn enforce_leading(node: Instance, leading: Combinator, scope: Instance) -> bool {
    match leading {
        // The leftmost match must be a direct child of the scope.
        Combinator::Child => node.get_parent() == Some(scope),
        // Already guaranteed to be a strict descendant of the scope by
        // construction (the candidate set and ancestor walks never escape it).
        Combinator::Descendant => true,
    }
}

#[cfg(test)]
mod tests {
    use super::compare_value;
    use rbx_dom_weak::types::{Variant as DomValue, Vector3 as DomVector3};

    #[test]
    fn bool_comparison() {
        assert_eq!(compare_value(&DomValue::Bool(true), "true"), Ok(true));
        assert_eq!(compare_value(&DomValue::Bool(true), "false"), Ok(false));
        assert_eq!(compare_value(&DomValue::Bool(false), "false"), Ok(true));
        // Only "true"/"false" are accepted for a boolean.
        assert_eq!(compare_value(&DomValue::Bool(true), "1"), Ok(false));
    }

    #[test]
    fn number_coercion() {
        assert_eq!(compare_value(&DomValue::Float64(4.0), "4"), Ok(true));
        assert_eq!(compare_value(&DomValue::Float64(4.0), "4.0"), Ok(true));
        assert_eq!(compare_value(&DomValue::Int32(1000), "1e3"), Ok(true));
        assert_eq!(compare_value(&DomValue::Int64(-5), "-5"), Ok(true));
        // A non-numeric literal never matches a number.
        assert_eq!(compare_value(&DomValue::Float64(4.0), "true"), Ok(false));
    }

    #[test]
    fn string_comparison() {
        assert_eq!(
            compare_value(&DomValue::String("Fuji".into()), "Fuji"),
            Ok(true)
        );
        // Case-sensitive.
        assert_eq!(
            compare_value(&DomValue::String("Fuji".into()), "fuji"),
            Ok(false)
        );
        // A numeric-looking literal compared to a string value is a string compare.
        assert_eq!(compare_value(&DomValue::String("4".into()), "4"), Ok(true));
    }

    #[test]
    fn unsupported_type_reports_name() {
        let value = DomValue::Vector3(DomVector3::new(1.0, 2.0, 3.0));
        assert_eq!(compare_value(&value, "1"), Err("Vector3".to_string()));
    }
}
