use thiserror::Error;

/**
    An error produced while parsing or evaluating a `QueryDescendants` selector.

    The messages for the confirmed cases (`PropertyExpectedEquals`,
    `ExpectedPropertyValue`, `ExpectedCloseBracket`, and the unsupported-type
    errors) are matched to the exact strings produced by the live Roblox engine.
*/
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum QueryError {
    #[error("'=' expected after property name")]
    PropertyExpectedEquals,
    #[error("Expected identifier for a property value")]
    ExpectedPropertyValue,
    #[error("Expected ']' to complete a property filter")]
    ExpectedCloseBracket,
    #[error("Expected a tag, class, name, or attribute selector")]
    ExpectedSelector,
    #[error("Expected an identifier")]
    ExpectedIdentifier,
    #[error("Unexpected trailing character in selector")]
    UnexpectedToken,
    #[error("Expected ')' to complete a pseudo-class selector")]
    ExpectedCloseParen,
    #[error("Unsupported pseudo-class ':{0}' (only ':not' and ':has' are supported)")]
    UnknownPseudoClass(String),
    #[error("Expected '(' after pseudo-class ':{0}'")]
    PseudoExpectedParen(String),
    #[error("Expected '\"' to complete a quoted value")]
    UnterminatedString,
    #[error("Property {name} of type {ty} is not supported for comparison")]
    UnsupportedPropertyType { name: String, ty: String },
    #[error("Attribute {name} of type {ty} is not supported for comparison")]
    UnsupportedAttributeType { name: String, ty: String },
}

/**
    Result type alias for any result returning `QueryError` in its error branch.
*/
pub type QueryResult<T> = Result<T, QueryError>;
