use nom::{branch::alt, bytes::complete::tag, combinator::map, IResult};

use super::util::*;
use crate::ast::Expression;

type ResultExpr<'a> = IResult<&'a str, Expression>;

pub fn parse_exp(input: &str) -> ResultExpr {
    alt((parse_nil, parse_false, parse_true))(input)
}

fn parse_nil(input: &str) -> ResultExpr {
    map(ws(tag("nil")), |_| Expression::Nil)(input)
}

fn parse_false(input: &str) -> ResultExpr {
    map(ws(tag("false")), |_| Expression::False)(input)
}

fn parse_true(input: &str) -> ResultExpr {
    map(ws(tag("true")), |_| Expression::True)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: write tests
    // My guess is that we probably want to write tests that
    // exercise parse_exp only, instead of having individual tests
    // for every helper parser. In other words, test the interface,
    // not the private functions

    #[test]
    fn accepts_nil() {
        let result = parse_exp("nil");
        assert_eq!(result, Ok(("", Expression::Nil)));

        let result = parse_exp("    nil  ");
        assert_eq!(result, Ok(("", Expression::Nil)));
    }

    #[test]
    fn accepts_bools() {
        let result = parse_exp("true");
        assert_eq!(result, Ok(("", Expression::True)));

        let result = parse_exp("false");
        assert_eq!(result, Ok(("", Expression::False)));

        let result = parse_exp("    true  ");
        assert_eq!(result, Ok(("", Expression::True)));
    }
}
