use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, i64},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

use super::{
    common::{parse_parlist, parse_var},
    parse_block,
    statement::parse_functioncall,
    util::*,
    ParseResult,
};
use crate::ast::{BinOp, Block, Expression, Field, Numeral, ParList, PrefixExp, UnOp};

pub fn parse_exp(input: &str) -> ParseResult<Expression> {
    alt((
        parse_nil,
        parse_false,
        parse_true,
        parse_binop_exp,
        parse_unary_op_exp,
        parse_numeral,
        parse_literal_string,
        parse_dot_dot_dot,
        // parse_fn_def,
        // map(parse_prefixexp, |result| {
        //     Expression::PrefixExp(Box::new(result))
        // }),
        parse_table_constructor,
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
    map(ws(float), |result| {
        Expression::Numeral(Numeral::Float(result.parse().unwrap()))
    })(input)
}

fn parse_literal_string(input: &str) -> ParseResult<Expression> {
    // LiteralString(String),
    // TODO(?): I'm ignoring string literals that aren't in double quotes for now
    map(ws(parse_string), |s| Expression::LiteralString(s))(input)
}

pub fn parse_dot_dot_dot(input: &str) -> ParseResult<Expression> {
    // DotDotDot, // Used for a variable number of arguments in things like functions
    map(ws(tag("...")), |_| Expression::DotDotDot)(input)
}

fn parse_fn_def(input: &str) -> ParseResult<Expression> {
    // FunctionDef((ParList, Block)),
    map(preceded(ws(tag("function")), parse_funcbody), |result| {
        Expression::FunctionDef(result)
    })(input)
}

fn parse_funcbody(input: &str) -> ParseResult<(ParList, Block)> {
    terminated(
        pair(delimited(char('('), parse_parlist, char(')')), parse_block),
        ws(tag("end")),
    )(input)
}

fn parse_prefixexp(input: &str) -> ParseResult<PrefixExp> {
    // PrefixExp(Box<PrefixExp>),
    alt((
        map(parse_var, |var| PrefixExp::Var(var)),
        map(parse_functioncall, |fncall| PrefixExp::FunctionCall(fncall)),
        map(delimited(ws(char('(')), parse_exp, ws(char(')'))), |exp| {
            PrefixExp::Exp(exp)
        }),
    ))(input)
}

fn parse_field(input: &str) -> ParseResult<Field> {
    let result = alt((
        map(
            separated_pair(
                delimited(ws(char('[')), parse_exp, ws(char(']'))),
                ws(char('=')),
                parse_exp,
            ),
            |result| Field::BracketedAssign(result),
        ),
        map(
            separated_pair(ws(identifier), ws(char('=')), ws(parse_exp)),
            |result| Field::NameAssign((String::from(result.0), result.1)),
        ),
        map(parse_exp, |result| Field::UnnamedAssign(result)),
    ))(input);

    result
}

fn parse_fieldlist(input: &str) -> ParseResult<Vec<Field>> {
    separated_list1(ws(alt((char(','), char(';')))), parse_field)(input)
}

fn parse_table_constructor(input: &str) -> ParseResult<Expression> {
    // TableConstructor(Vec<Field>),
    map(
        delimited(ws(char('{')), opt(parse_fieldlist), ws(char('}'))),
        |result| match result {
            Some(fields) => Expression::TableConstructor(fields),
            None => Expression::TableConstructor(Vec::new()),
        },
    )(input)
}

fn parse_binop(input: &str) -> ParseResult<BinOp> {
    ws(alt((
        map(tag("+"), |_| BinOp::Add),
        map(tag("-"), |_| BinOp::Sub),
        map(tag("*"), |_| BinOp::Mult),
        map(tag("/"), |_| BinOp::Div),
        map(tag("//"), |_| BinOp::IntegerDiv),
        map(tag("^"), |_| BinOp::Pow),
        map(tag("%"), |_| BinOp::Mod),
        map(tag("&"), |_| BinOp::BitAnd),
        map(tag("~"), |_| BinOp::BitXor),
        map(tag("|"), |_| BinOp::BitOr),
        map(tag(">>"), |_| BinOp::ShiftRight),
        map(tag("<<"), |_| BinOp::ShiftLeft),
        map(tag(".."), |_| BinOp::Concat),
        map(tag("<"), |_| BinOp::LessThan),
        map(tag("<="), |_| BinOp::LessEq),
        map(tag(">"), |_| BinOp::GreaterThan),
        map(tag(">="), |_| BinOp::GreaterEq),
        map(tag("=="), |_| BinOp::Equal),
        map(tag("~="), |_| BinOp::NotEqual),
        map(tag("and"), |_| BinOp::LogicalAnd),
        map(tag("or"), |_| BinOp::LogicalOr),
    )))(input)
}

fn parse_binop_exp(input: &str) -> ParseResult<Expression> {
    // BinaryOp((Box<Expression>, BinOp, Box<Expression>)),
    map(tuple((parse_exp, parse_binop, parse_exp)), |result| {
        Expression::BinaryOp((Box::new(result.0), result.1, Box::new(result.2)))
    })(input)
}

fn parse_unary_op(input: &str) -> ParseResult<UnOp> {
    // UnaryOp((UnOp, Box<Expression>)),
    ws(alt((
        map(tag("-"), |_| UnOp::Negate),
        map(tag("not"), |_| UnOp::LogicalNot),
        map(tag("#"), |_| UnOp::Length),
        map(tag("~"), |_| UnOp::BitNot),
    )))(input)
}

fn parse_unary_op_exp(input: &str) -> ParseResult<Expression> {
    // UnaryOp((UnOp, Box<Expression>)),
    map(tuple((parse_unary_op, parse_exp)), |result| {
        Expression::UnaryOp((result.0, Box::new(result.1)))
    })(input)
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

    #[test]
    fn accepts_dotdotdot() {
        let result = parse_exp("  ...  ");
        assert_eq!(result, Ok(("", Expression::DotDotDot)));
    }

    #[test]
    fn accepts_tableconstructor() {
        let input = "{red=\"#ff0000\",[1]=3.14159265,true}";

        let expected = Ok((
            "",
            Expression::TableConstructor(vec![
                Field::NameAssign((
                    String::from("red"),
                    Expression::LiteralString(String::from("#ff0000")),
                )),
                Field::BracketedAssign((
                    Expression::Numeral(Numeral::Integer(1)),
                    Expression::Numeral(Numeral::Float(3.14159265)),
                )),
                Field::UnnamedAssign(Expression::True),
            ]),
        ));

        let actual = parse_exp(input);
        assert_eq!(actual, expected);

        let input = "{
            red=\"#ff0000\",
            [1]=3.14159265,
            true
        }";

        let actual = parse_exp(input);
        assert_eq!(actual, expected);

        let input = "{
            red      =       \"#ff0000\"       ,
            [  1      ]  =    3.14159265    ,
            true
        }";

        let actual = parse_exp(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn accepts_binop_exp() {
        let input = "1 + 2";
        let result = parse_exp(input);

        assert_eq!(
            result,
            Ok((
                "",
                Expression::BinaryOp((
                    Box::new(Expression::Numeral(Numeral::Integer(1))),
                    BinOp::Add,
                    Box::new(Expression::Numeral(Numeral::Integer(2)))
                ))
            ))
        );
    }
}
