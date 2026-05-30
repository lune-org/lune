/*!
    Abstract syntax tree for the `QueryDescendants` selector grammar.

    The hierarchy mirrors the grammar: a [`SelectorList`] (the `,` operator) of
    [`ComplexSelector`]s (combinator-joined [`CompoundSelector`]s), each compound
    being a logical AND of [`SimpleSelector`]s.
*/

/**
    A comma-separated list of independent complex selectors (the `,` operator).
*/
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SelectorList(pub Vec<ComplexSelector>);

/**
    A combinator connecting two compound selectors, or the leftmost compound to
    the current scope.

    `Descendant` (`>>`, and the implicit default) matches any ancestor up to, but
    not including, the current scope; `Child` (`>`) matches only the exact parent
    within the scope.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Combinator {
    Descendant,
    Child,
}

/**
    A complex selector: a leading combinator connecting the leftmost compound to
    the current scope, followed by one or more compounds joined by combinators.

    Stored left-to-right but matched right-to-left. `parts[i].combinator` connects
    `parts[i - 1]` to `parts[i]`, so `parts[0].combinator` is unused - the leftmost
    compound is connected to the scope by `leading` instead.
*/
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ComplexSelector {
    pub leading: Combinator,
    pub parts: Vec<ComplexPart>,
}

/**
    A single compound in a complex selector, together with the combinator that
    connects it to the compound on its left.
*/
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ComplexPart {
    pub combinator: Combinator,
    pub compound: CompoundSelector,
}

/**
    One or more simple selectors, all of which must match the same instance.
*/
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CompoundSelector(pub Vec<SimpleSelector>);

/**
    A single term of a compound selector.

    Each variant corresponds to one piece of selector syntax: a class name
    (matched via `IsA`), a `.Tag`, a `#Name`, a `[Prop = Value]` property, an
    `[$Attr]` attribute presence test or `[$Attr = Value]` attribute value test,
    or a `:not(...)` / `:has(...)` pseudo-class.
*/
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SimpleSelector {
    Type(String),
    Tag(String),
    Name(String),
    Property { name: String, value: String },
    AttributeExists { name: String },
    Attribute { name: String, value: String },
    Not(SelectorList),
    Has(SelectorList),
}
