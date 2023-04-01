use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, i64},
    combinator::map,
    number::complete::double,
    sequence::{delimited, pair, preceded, terminated},
};

use super::{
    common::{parse_parlist, parse_var},
    parse_block,
    statement::parse_functioncall,
    util::*,
    ParseResult,
};
use crate::ast::{Block, Expression, Numeral, ParList, PrefixExp};

pub fn parse_exp(input: &str) -> ParseResult<Expression> {
    alt((
        parse_nil,
        parse_false,
        parse_true,
        parse_numeral,
        parse_literal_string,
    ))(input)
}

fn parse_nil(input: &str) -> ParseResult<Expression> {
    map(ws(tag("nil")), |_| Expression::Nil)(input)
}

fn parse_false(input: &str) -> ParseResult<Expression> {
    map(ws(tag("false")), |_| Expression::False)(input)
}

fn parse_true(input: &str) -> ParseResult<Expression> {
    map(ws(tag("true")), |_| Expression::True)(input)
}

fn parse_numeral(input: &str) -> ParseResult<Expression> {
    // TODO: other formats of floats
    alt((parse_float, parse_integer))(input)
}

fn parse_integer(input: &str) -> ParseResult<Expression> {
    map(ws(i64), |numeral: i64| {
        Expression::Numeral(Numeral::Integer(numeral))
    })(input)
}

fn parse_float(input: &str) -> ParseResult<Expression> {
    map(ws(double), |numeral: f64| {
        Expression::Numeral(Numeral::Float(numeral))
    })(input)
}

fn parse_literal_string(input: &str) -> ParseResult<Expression> {
    // LiteralString(String),
    // TODO(?): I'm ignoring string literals that aren't in double quotes for now
    map(ws(parse_string), |s| Expression::LiteralString(s))(input)
}

pub fn parse_dot_dot_dot(input: &str) -> ParseResult<Expression> {
    // DotDotDot, // Used for a variable number of arguments in things like functions
    map(tag("..."), |_| Expression::DotDotDot)(input)
}

fn parse_fn_def(input: &str) -> ParseResult<Expression> {
    // FunctionDef((ParList, Block)),
    map(preceded(ws(tag("function")), parse_funcbody), |result| {
        Expression::FunctionDef(result)
    })(input)
}

pub fn parse_funcbody(input: &str) -> ParseResult<(ParList, Block)> {
    terminated(
        pair(delimited(char('('), parse_parlist, char(')')), parse_block),
        ws(tag("end")),
    )(input)
}

fn parse_prefix_exp(input: &str) -> ParseResult<PrefixExp> {
    // PrefixExp(Box<PrefixExp>),
    alt((
        map(parse_var, |var| PrefixExp::Var(var)),
        map(parse_functioncall, |fncall| PrefixExp::FunctionCall(fncall)),
        map(delimited(ws(char('(')), parse_exp, ws(char(')'))), |exp| {
            PrefixExp::Exp(exp)
        }),
    ))(input)
}

fn parse_table_constructor(input: &str) -> ParseResult<Expression> {
    // TableConstructor(Vec<Field>),
    unimplemented!()
}

fn parse_binary_op(input: &str) -> ParseResult<Expression> {
    // BinaryOp((Box<Expression>, BinOp, Box<Expression>)),
    unimplemented!()
}

fn parse_unary_op(input: &str) -> ParseResult<Expression> {
    // UnaryOp((UnOp, Box<Expression>)),
    unimplemented!()
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

    #[test]
    fn accepts_numerals() {
        let result = parse_exp("123");
        assert_eq!(result, Ok(("", Expression::Numeral(Numeral::Integer(123)))));

        let result = parse_exp("    -123  ");
        assert_eq!(
            result,
            Ok(("", Expression::Numeral(Numeral::Integer(-123))))
        );

        let result = parse_exp("    1.23  ");
        assert_eq!(result, Ok(("", Expression::Numeral(Numeral::Float(1.23)))));

        let result = parse_exp("    -1.23e-4  ");
        assert_eq!(
            result,
            Ok(("", Expression::Numeral(Numeral::Float(-1.23e-4))))
        );
    }

    #[test]
    fn accepts_string_literals() {
        let result = parse_exp("    \"Foo bar baz!\"     ");
        assert_eq!(
            result,
            Ok(("", Expression::LiteralString(String::from("Foo bar baz!"))))
        );

        let data = "\"tab:\\tafter tab, newline:\\nnew line, quote: \\\", emoji: \\u{1F602}, newline:\\nescaped whitespace: \\    abc\"";
        let expected = "tab:\tafter tab, newline:\nnew line, quote: \", emoji: 😂, newline:\nescaped whitespace: abc";
        let result = parse_exp(data);
        assert_eq!(
            result,
            Ok(("", Expression::LiteralString(String::from(expected))))
        );
    }
}
