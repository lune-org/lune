/*!
    Implementation of the selector engine behind
    [`Instance:QueryDescendants`](https://create.roblox.com/docs/reference/engine/classes/Instance#QueryDescendants).

    The selector grammar is a variant of the one used by `StyleRule`, verified
    empirically against the live Roblox engine. It consists of:

    - Simple selectors: `ClassName` (matched via `IsA`), `.Tag`
      (`CollectionService` tag), `#Name` (exact `Instance.Name`),
      `[Prop = Value]`, `[$Attr]` (attribute presence), `[$Attr = Value]`.
    - Compound selectors: one or more simple selectors written adjacent (any
      whitespace between them is insignificant); all must match the same
      instance (logical AND).
    - Combinators: `>` (direct child), `>>` (descendant - also the implicit
      default), and `,` (independent selector list / union).
    - Pseudo-classes: `:not(<selector-list>)` and `:has(<relative-selector-list>)`.

    The engine is split into the [`lexer`], [`parser`] (producing the [`ast`]),
    and [`matcher`]. It is intentionally free of any `mlua` dependency so it can
    be unit-tested in isolation; only the `QueryDescendants` method shim in
    `base.rs` touches lua.
*/

pub(crate) mod ast;
mod error;
mod lexer;
mod matcher;
mod parser;

#[cfg(test)]
mod tests;

use super::Instance;

pub use self::error::{QueryError, QueryResult};

/**
    Runs a `QueryDescendants` selector against the descendants of `root`.

    Returns the matching descendants in preorder depth-first order. For a
    selector list (the `,` operator) the matches of each complex selector
    are appended in order and are NOT de-duplicated, mirroring Roblox.

    # Errors

    Returns a [`QueryError`] if the selector is malformed, or if it compares
    a property / attribute whose type does not support comparison.
*/
pub fn query_descendants(root: Instance, selector: impl AsRef<str>) -> QueryResult<Vec<Instance>> {
    let selector: &str = selector.as_ref();
    let list = parser::parse(selector)?;

    let descendants = root.get_descendants_preorder();
    let mut results = Vec::new();
    for complex in &list.0 {
        for node in &descendants {
            if matcher::matches_complex(*node, complex, root)? {
                results.push(*node);
            }
        }
    }

    Ok(results)
}
