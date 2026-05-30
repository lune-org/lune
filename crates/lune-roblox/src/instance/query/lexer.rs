/*!
    Lexer for the `QueryDescendants` selector grammar.

    Whitespace is significant only as a token separator (never as a combinator);
    it is not emitted as a token. The `>>` (descendant) combinator wins over `>`
    (child) via maximal munch.
*/

use super::QueryError;

/**
    A single lexical token of a selector string.
*/
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Ident(String),
    Str(String),
    Hash,
    Dot,
    Dollar,
    LBracket,
    RBracket,
    Eq,
    Comma,
    Child,
    Descendant,
    Colon,
    LParen,
    RParen,
    // Any character not recognized as part of another token. Kept (rather than
    // erroring during lexing) so the parser can produce a context-appropriate
    // error - e.g. a stray `'` where a value is expected becomes a
    // `QueryError::ExpectedPropertyValue`.
    Unknown,
}

pub(crate) fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

pub(crate) fn tokenize(input: &str) -> Result<Vec<Token>, QueryError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '#' => tokens.push(Token::Hash),
            '.' => tokens.push(Token::Dot),
            '$' => tokens.push(Token::Dollar),
            '[' => tokens.push(Token::LBracket),
            ']' => tokens.push(Token::RBracket),
            '=' => tokens.push(Token::Eq),
            ',' => tokens.push(Token::Comma),
            ':' => tokens.push(Token::Colon),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '>' => {
                // Maximal munch: ">>" must win over ">". Whitespace is never a
                // token, so "> >" naturally lexes as two separate child
                // combinators rather than a descendant combinator.
                if chars.get(i + 1) == Some(&'>') {
                    tokens.push(Token::Descendant);
                    i += 2;
                    continue;
                }
                tokens.push(Token::Child);
            }
            '"' => {
                let mut value = String::new();
                i += 1;
                let mut closed = false;
                while i < chars.len() {
                    if chars[i] == '"' {
                        closed = true;
                        i += 1;
                        break;
                    }
                    value.push(chars[i]);
                    i += 1;
                }
                if !closed {
                    return Err(QueryError::UnterminatedString);
                }
                tokens.push(Token::Str(value));
                continue;
            }
            c if is_ident_char(c) => {
                let start = i;
                while i < chars.len() && is_ident_char(chars[i]) {
                    i += 1;
                }
                tokens.push(Token::Ident(chars[start..i].iter().collect()));
                continue;
            }
            _ => tokens.push(Token::Unknown),
        }
        i += 1;
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_simple_selectors() {
        assert_eq!(
            tokenize("Model.Apple").unwrap(),
            vec![
                Token::Ident("Model".into()),
                Token::Dot,
                Token::Ident("Apple".into())
            ]
        );
        assert_eq!(
            tokenize("[$Attr = 4]").unwrap(),
            vec![
                Token::LBracket,
                Token::Dollar,
                Token::Ident("Attr".into()),
                Token::Eq,
                Token::Ident("4".into()),
                Token::RBracket
            ]
        );
    }

    #[test]
    fn maximal_munch_descendant() {
        assert_eq!(
            tokenize("A>>B").unwrap(),
            vec![
                Token::Ident("A".into()),
                Token::Descendant,
                Token::Ident("B".into())
            ]
        );
        // Whitespace splits ">>" into two child combinators.
        assert_eq!(tokenize("> >").unwrap(), vec![Token::Child, Token::Child]);
    }

    #[test]
    fn whitespace_is_not_a_token() {
        assert_eq!(
            tokenize("  Model   Part  ").unwrap(),
            vec![Token::Ident("Model".into()), Token::Ident("Part".into())]
        );
    }

    #[test]
    fn quoted_string_and_unterminated() {
        assert_eq!(
            tokenize(r#""Red Tree""#).unwrap(),
            vec![Token::Str("Red Tree".into())]
        );
        assert_eq!(tokenize(r#""oops"#), Err(QueryError::UnterminatedString));
    }

    #[test]
    fn unknown_characters() {
        assert_eq!(tokenize("*").unwrap(), vec![Token::Unknown]);
        assert_eq!(tokenize("'").unwrap(), vec![Token::Unknown]);
    }
}
