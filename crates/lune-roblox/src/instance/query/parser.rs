/*!
    Recursive-descent parser for the `QueryDescendants` selector grammar.
*/

use super::QueryError;
use super::ast::{
    Combinator, ComplexPart, ComplexSelector, CompoundSelector, SelectorList, SimpleSelector,
};
use super::lexer::{Token, tokenize};

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn eat(&mut self, token: &Token) -> bool {
        if self.peek() == Some(token) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn peek_combinator(&self) -> Option<Combinator> {
        match self.peek() {
            Some(Token::Child) => Some(Combinator::Child),
            Some(Token::Descendant) => Some(Combinator::Descendant),
            _ => None,
        }
    }

    fn eat_combinator(&mut self) -> Option<Combinator> {
        let combinator = self.peek_combinator();
        if combinator.is_some() {
            self.pos += 1;
        }
        combinator
    }

    fn parse_selector_list(&mut self) -> Result<SelectorList, QueryError> {
        let mut selectors = vec![self.parse_complex()?];
        while self.eat(&Token::Comma) {
            selectors.push(self.parse_complex()?);
        }
        Ok(SelectorList(selectors))
    }

    fn parse_complex(&mut self) -> Result<ComplexSelector, QueryError> {
        let leading = self.eat_combinator().unwrap_or(Combinator::Descendant);

        let first = self.parse_compound()?;
        let mut parts = vec![ComplexPart {
            combinator: Combinator::Descendant, // unused for parts[0]
            compound: first,
        }];

        while let Some(combinator) = self.eat_combinator() {
            let compound = self.parse_compound()?;
            parts.push(ComplexPart {
                combinator,
                compound,
            });
        }

        Ok(ComplexSelector { leading, parts })
    }

    fn parse_compound(&mut self) -> Result<CompoundSelector, QueryError> {
        let mut simples = Vec::new();
        while matches!(
            self.peek(),
            Some(Token::Ident(_) | Token::Dot | Token::Hash | Token::LBracket | Token::Colon)
        ) {
            simples.push(self.parse_simple()?);
        }
        if simples.is_empty() {
            return Err(QueryError::ExpectedSelector);
        }
        Ok(CompoundSelector(simples))
    }

    fn parse_ident(&mut self) -> Result<String, QueryError> {
        match self.advance() {
            Some(Token::Ident(name)) => Ok(name.clone()),
            _ => Err(QueryError::ExpectedIdentifier),
        }
    }

    fn parse_simple(&mut self) -> Result<SimpleSelector, QueryError> {
        match self.advance() {
            Some(Token::Ident(name)) => Ok(SimpleSelector::Type(name.clone())),
            Some(Token::Dot) => Ok(SimpleSelector::Tag(self.parse_ident()?)),
            Some(Token::Hash) => Ok(SimpleSelector::Name(self.parse_ident()?)),
            Some(Token::LBracket) => self.parse_bracket(),
            Some(Token::Colon) => self.parse_pseudo(),
            _ => Err(QueryError::ExpectedSelector),
        }
    }

    fn parse_value(&mut self) -> Result<String, QueryError> {
        match self.advance() {
            Some(Token::Ident(value) | Token::Str(value)) => Ok(value.clone()),
            _ => Err(QueryError::ExpectedPropertyValue),
        }
    }

    fn expect_close_bracket(&mut self) -> Result<(), QueryError> {
        if self.eat(&Token::RBracket) {
            Ok(())
        } else {
            Err(QueryError::ExpectedCloseBracket)
        }
    }

    fn parse_bracket(&mut self) -> Result<SimpleSelector, QueryError> {
        // Already consumed the opening '['.
        if self.eat(&Token::Dollar) {
            // Attribute: presence "[$Attr]" or equality "[$Attr = Value]".
            let name = self.parse_ident()?;
            if self.eat(&Token::Eq) {
                let value = self.parse_value()?;
                self.expect_close_bracket()?;
                Ok(SimpleSelector::Attribute { name, value })
            } else {
                self.expect_close_bracket()?;
                Ok(SimpleSelector::AttributeExists { name })
            }
        } else {
            // Property: equality only. There is no bare "[Prop]" presence form.
            let name = self.parse_ident()?;
            if !self.eat(&Token::Eq) {
                return Err(QueryError::PropertyExpectedEquals);
            }
            let value = self.parse_value()?;
            self.expect_close_bracket()?;
            Ok(SimpleSelector::Property { name, value })
        }
    }

    fn parse_pseudo(&mut self) -> Result<SimpleSelector, QueryError> {
        // Already consumed the ':'.
        let kind = self.parse_ident()?;
        if !self.eat(&Token::LParen) {
            return Err(QueryError::PseudoExpectedParen(kind));
        }
        let list = self.parse_selector_list()?;
        if !self.eat(&Token::RParen) {
            return Err(QueryError::ExpectedCloseParen);
        }
        match kind.as_str() {
            "not" => Ok(SimpleSelector::Not(list)),
            "has" => Ok(SimpleSelector::Has(list)),
            _ => Err(QueryError::UnknownPseudoClass(kind)),
        }
    }
}

pub(crate) fn parse(input: &str) -> Result<SelectorList, QueryError> {
    let tokens = tokenize(input)?;
    // An empty or whitespace-only selector matches nothing (Roblox returns an
    // empty array rather than erroring).
    if tokens.is_empty() {
        return Ok(SelectorList(Vec::new()));
    }
    let mut parser = Parser { tokens, pos: 0 };
    let list = parser.parse_selector_list()?;
    if parser.pos != parser.tokens.len() {
        return Err(QueryError::UnexpectedToken);
    }
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::instance::query::QueryError;
    use crate::instance::query::ast::{Combinator, SimpleSelector};

    fn parse_ok(input: &str) -> super::SelectorList {
        parse(input).unwrap_or_else(|e| panic!("expected `{input}` to parse, got error: {e}"))
    }

    #[test]
    fn empty_is_empty_list() {
        assert_eq!(parse_ok("").0.len(), 0);
        assert_eq!(parse_ok("   ").0.len(), 0);
    }

    #[test]
    fn simple_selectors() {
        assert_eq!(
            parse_ok("MeshPart").0[0].parts[0].compound.0,
            vec![SimpleSelector::Type("MeshPart".into())]
        );
        assert_eq!(
            parse_ok(".Fruit").0[0].parts[0].compound.0,
            vec![SimpleSelector::Tag("Fruit".into())]
        );
        assert_eq!(
            parse_ok("#RedTree").0[0].parts[0].compound.0,
            vec![SimpleSelector::Name("RedTree".into())]
        );
    }

    #[test]
    fn compound_and_combinators() {
        // Whitespace-joined simples form one compound.
        let ws = parse_ok("Part #RedTree");
        assert_eq!(ws.0[0].parts.len(), 1);
        assert_eq!(ws.0[0].parts[0].compound.0.len(), 2);

        // An explicit ">>" splits into two compounds.
        let desc = parse_ok("Model >> Part");
        assert_eq!(desc.0[0].parts.len(), 2);
        assert_eq!(desc.0[0].parts[1].combinator, Combinator::Descendant);

        // Leading combinators.
        assert_eq!(parse_ok("> Part").0[0].leading, Combinator::Child);
        assert_eq!(parse_ok("Part").0[0].leading, Combinator::Descendant);
    }

    #[test]
    fn brackets_and_pseudo() {
        assert_eq!(
            parse_ok("[CanCollide = false]").0[0].parts[0].compound.0,
            vec![SimpleSelector::Property {
                name: "CanCollide".into(),
                value: "false".into()
            }]
        );
        assert_eq!(
            parse_ok("[$FuelCapacity]").0[0].parts[0].compound.0,
            vec![SimpleSelector::AttributeExists {
                name: "FuelCapacity".into()
            }]
        );
        // Quoted values strip the quotes; a space inside is preserved.
        assert_eq!(
            parse_ok(r#"[Name = "Red Tree"]"#).0[0].parts[0].compound.0,
            vec![SimpleSelector::Property {
                name: "Name".into(),
                value: "Red Tree".into()
            }]
        );
        assert!(matches!(
            parse_ok(":not(MeshPart)").0[0].parts[0].compound.0[0],
            SimpleSelector::Not(_)
        ));
        assert!(matches!(
            parse_ok("MeshPart:has(> .SwordPart)").0[0].parts[0]
                .compound
                .0[1],
            SimpleSelector::Has(_)
        ));
    }

    #[test]
    fn errors_match_roblox() {
        // Confirmed exact Roblox error strings.
        assert_eq!(parse("[Prop]"), Err(QueryError::PropertyExpectedEquals));
        assert_eq!(
            parse("[Name = 'x']"),
            Err(QueryError::ExpectedPropertyValue)
        );
        assert_eq!(parse("[Foo = 4.0]"), Err(QueryError::ExpectedCloseBracket));
        assert_eq!(
            parse("[Transparency = 0.5]"),
            Err(QueryError::ExpectedCloseBracket)
        );
    }

    #[test]
    fn errors_other() {
        assert!(parse("A,").is_err());
        assert!(parse(",A").is_err());
        assert!(parse("Model >").is_err());
        assert!(parse(">>").is_err());
        assert!(parse(":Hover").is_err());
        assert!(parse(":foo(Part)").is_err());
        assert!(parse("*").is_err());
        assert!(parse("Part + Part").is_err());
        assert!(parse(":not(SpotLight").is_err());
        assert!(parse("[CanCollide = false").is_err());
        assert!(parse(r#"[$X = "unterminated]"#).is_err());
    }
}
