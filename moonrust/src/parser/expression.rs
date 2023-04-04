use std::{iter, result};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, i64},
    combinator::{map, opt},
    multi::{many0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Parser,
};

use super::{
    common::{parse_parlist, parse_var},
    parse_block,
    statement::parse_functioncall,
    util::*,
    ParseResult,
};
use crate::ast::{BinOp, Block, Expression, Field, Numeral, ParList, PrefixExp, UnOp};

// pub fn parse_exp(input: &str) -> ParseResult<Expression> {
//     alt((
//         parse_nil,
//         parse_false,
//         parse_true,
//         parse_binop_exp,
//         parse_unary_op_exp,
//         parse_numeral,
//         parse_literal_string,
//         parse_dot_dot_dot,
//         // parse_fn_def,
//         // map(parse_prefixexp, |result| {
//         //     Expression::PrefixExp(Box::new(result))
//         // }),
//         parse_table_constructor_exp,
//     ))(input)
// }

pub fn parse_exp(input: &str) -> ParseResult<Expression> {
    unimplemented!()
}

fn parse_or_exp(input: &str) -> ParseResult<Expression> {
    unimplemented!()
}

fn parse_and_exp(input: &str) -> ParseResult<Expression> {
    unimplemented!()
}
// ?
fn parse_rel_exp(input: &str) -> ParseResult<Expression> {
    unimplemented!()
}

fn parse_concat_expr(input: &str) -> ParseResult<Expression> {
    map(
        pair(parse_add_exp, many0(preceded(ws(tag("..")), parse_add_exp))),
        |result| foldr_op_exp(result.0, BinOp::Concat, result.1),
    )(input)
}

fn parse_add_exp(input: &str) -> ParseResult<Expression> {
    map(
        pair(parse_mult_exp, many0(pair(parse_add_op, parse_mult_exp))),
        |result| fold_exp(result.0, result.1),
    )(input)
}

fn parse_add_op(input: &str) -> ParseResult<BinOp> {
    ws(alt((
        map(char('+'), |_| BinOp::Add),
        map(char('-'), |_| BinOp::Sub),
    )))(input)
}

fn parse_mult_exp(input: &str) -> ParseResult<Expression> {
    map(
        pair(parse_unary_exp, many0(pair(parse_mult_op, parse_unary_exp))),
        |result| fold_exp(result.0, result.1),
    )(input)
}

/// Fold (in a left-associative manner) a list of binary operators and expressions into a single expression.
/// An initial expression must be given to start the fold.
fn fold_exp(init: Expression, op_and_exps: Vec<(BinOp, Expression)>) -> Expression {
    op_and_exps.into_iter().fold(init, |acc, op_and_exp| {
        Expression::BinaryOp((Box::new(acc), op_and_exp.0, Box::new(op_and_exp.1)))
    })
}

/// Fold (in a right-associative manner) a list of expressions given an initial expression and a binary operator.
fn foldr_op_exp(init: Expression, op: BinOp, exps: Vec<Expression>) -> Expression {
    iter::once(init)
        .chain(exps.into_iter())
        .rfold(None, |acc, exp| match acc {
            None => Some(exp),
            Some(acc_exp) => Some(Expression::BinaryOp((Box::new(exp), op, Box::new(acc_exp)))),
        })
        .unwrap() // We'll definitely have at least one element, so None is impossible
}

fn parse_mult_op(input: &str) -> ParseResult<BinOp> {
    ws(alt((
        map(char('*'), |_| BinOp::Mult),
        map(char('/'), |_| BinOp::Div),
        map(tag("//"), |_| BinOp::IntegerDiv),
        map(char('%'), |_| BinOp::Mod),
    )))(input)
}

fn parse_unary_exp(input: &str) -> ParseResult<Expression> {
    alt((
        map(preceded(ws(char('-')), parse_unary_exp), |result| {
            Expression::UnaryOp((UnOp::Negate, Box::new(result)))
        }),
        map(preceded(ws(tag("not")), parse_unary_exp), |result| {
            Expression::UnaryOp((UnOp::LogicalNot, Box::new(result)))
        }),
        map(preceded(ws(char('#')), parse_pow_exp), |result| {
            Expression::UnaryOp((UnOp::Length, Box::new(result)))
        }),
        map(preceded(ws(char('~')), parse_unary_exp), |result| {
            Expression::UnaryOp((UnOp::BitNot, Box::new(result)))
        }),
        parse_pow_exp,
    ))(input)
}

fn parse_pow_exp(input: &str) -> ParseResult<Expression> {
    map(
        pair(parse_atom, many0(preceded(ws(char('^')), parse_atom))),
        |result| foldr_op_exp(result.0, BinOp::Pow, result.1),
    )(input)
}

/// 1 ^ (2 ^ (3 ^ 5))

fn parse_atom(input: &str) -> ParseResult<Expression> {
    alt((
        parse_nil,
        parse_true,
        parse_false,
        parse_numeral,
        parse_literal_string,
        parse_dot_dot_dot,
        parse_fn_def,
        map(parse_prefixexp, |result| {
            Expression::PrefixExp(Box::new(result))
        }),
        parse_table_constructor_exp,
    ))(input)
}

/// ------------------------------------------------------------------

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

pub fn parse_funcbody(input: &str) -> ParseResult<(ParList, Block)> {
    terminated(
        pair(delimited(char('('), parse_parlist, char(')')), parse_block),
        ws(tag("end")),
    )(input)
}

pub fn parse_prefixexp(input: &str) -> ParseResult<PrefixExp> {
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

pub fn parse_table_constructor(input: &str) -> ParseResult<Vec<Field>> {
    // TableConstructor(Vec<Field>),
    map(
        delimited(ws(char('{')), opt(parse_fieldlist), ws(char('}'))),
        |result| match result {
            Some(fields) => fields,
            None => Vec::new(),
        },
    )(input)
}

fn parse_table_constructor_exp(input: &str) -> ParseResult<Expression> {
    // TableConstructor(Vec<Field>),
    map(parse_table_constructor, |result| {
        Expression::TableConstructor(result)
    })(input)
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
