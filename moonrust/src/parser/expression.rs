use std::iter;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, i64},
    combinator::map,
    multi::many0,
    sequence::{pair, preceded},
};

use super::{
    common::{
        parse_dot_dot_dot, parse_funcbody, parse_literal_string, parse_prefixexp,
        parse_table_constructor,
    },
    util::*,
    ParseResult,
};
use crate::ast::{BinOp, Expression, Numeral, UnOp};

pub fn parse_exp(input: &str) -> ParseResult<Expression> {
    parse_or_exp(input)
}

fn parse_or_exp(input: &str) -> ParseResult<Expression> {
    map(
        pair(
            parse_and_exp,
            many0(pair(
                map(ws(tag("or")), |_| BinOp::LogicalOr),
                parse_and_exp,
            )),
        ),
        |result| foldl_exp(result.0, result.1),
    )(input)
}

fn parse_and_exp(input: &str) -> ParseResult<Expression> {
    map(
        pair(
            parse_rel_exp,
            many0(pair(
                map(ws(tag("and")), |_| BinOp::LogicalAnd),
                parse_rel_exp,
            )),
        ),
        |result| foldl_exp(result.0, result.1),
    )(input)
}

fn parse_rel_exp(input: &str) -> ParseResult<Expression> {
    fn parse_rel_op(input: &str) -> ParseResult<BinOp> {
        ws(alt((
            map(char('<'), |_| BinOp::LessThan),
            map(char('>'), |_| BinOp::GreaterThan),
            map(tag("<="), |_| BinOp::LessEq),
            map(tag(">="), |_| BinOp::GreaterEq),
            map(tag("~="), |_| BinOp::NotEqual),
            map(tag("=="), |_| BinOp::Equal),
        )))(input)
    }

    map(
        pair(
            parse_concat_expr,
            many0(pair(parse_rel_op, parse_concat_expr)),
        ),
        |result| foldl_exp(result.0, result.1),
    )(input)
}

fn parse_concat_expr(input: &str) -> ParseResult<Expression> {
    map(
        pair(parse_add_exp, many0(preceded(ws(tag("..")), parse_add_exp))),
        |result| foldr_op_exp(result.0, BinOp::Concat, result.1),
    )(input)
}

fn parse_add_exp(input: &str) -> ParseResult<Expression> {
    fn parse_add_op(input: &str) -> ParseResult<BinOp> {
        ws(alt((
            map(char('+'), |_| BinOp::Add),
            map(char('-'), |_| BinOp::Sub),
        )))(input)
    }

    map(
        pair(parse_mult_exp, many0(pair(parse_add_op, parse_mult_exp))),
        |result| foldl_exp(result.0, result.1),
    )(input)
}

fn parse_mult_exp(input: &str) -> ParseResult<Expression> {
    fn parse_mult_op(input: &str) -> ParseResult<BinOp> {
        ws(alt((
            map(char('*'), |_| BinOp::Mult),
            map(char('/'), |_| BinOp::Div),
            map(tag("//"), |_| BinOp::IntegerDiv),
            map(char('%'), |_| BinOp::Mod),
        )))(input)
    }

    map(
        pair(parse_unary_exp, many0(pair(parse_mult_op, parse_unary_exp))),
        |result| foldl_exp(result.0, result.1),
    )(input)
}

fn parse_unary_exp(input: &str) -> ParseResult<Expression> {
    alt((
        map(preceded(ws(char('-')), parse_unary_exp), |result| {
            Expression::UnaryOp((UnOp::Negate, Box::new(result)))
        }),
        map(preceded(ws(tag("not")), parse_unary_exp), |result| {
            Expression::UnaryOp((UnOp::LogicalNot, Box::new(result)))
        }),
        map(preceded(ws(char('#')), parse_unary_exp), |result| {
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

fn parse_atom(input: &str) -> ParseResult<Expression> {
    alt((
        parse_nil,
        parse_true,
        parse_false,
        parse_numeral,
        parse_literal_string,
        parse_dot_dot_dot,
        parse_fn_def,
        parse_table_constructor_exp,
        map(parse_prefixexp, |result| {
            Expression::PrefixExp(Box::new(result))
        }),
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

fn parse_fn_def(input: &str) -> ParseResult<Expression> {
    map(preceded(ws(tag("function")), parse_funcbody), |result| {
        Expression::FunctionDef(result)
    })(input)
}

fn parse_table_constructor_exp(input: &str) -> ParseResult<Expression> {
    map(parse_table_constructor, |result| {
        Expression::TableConstructor(result)
    })(input)
}

/// Fold (in a left-associative manner) a list of binary operators and expressions into a single expression.
/// An initial expression must be given to start the fold.
fn foldl_exp(init: Expression, op_and_exps: Vec<(BinOp, Expression)>) -> Expression {
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

#[cfg(test)]
mod tests {
    use crate::ast::{Args, Field, FunctionCall, PrefixExp, Var};

    use super::*;

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

        let result = parse_exp("    1.23  ");
        assert_eq!(result, Ok(("", Expression::Numeral(Numeral::Float(1.23)))));

        let result = parse_exp("    6.02E23  ");
        assert_eq!(
            result,
            Ok(("", Expression::Numeral(Numeral::Float(6.02E23))))
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

        // Test right associativity for pow
        let input = "2 ^ 3 ^ 4";
        let result = parse_exp(input);

        assert_eq!(
            result,
            Ok((
                "",
                Expression::BinaryOp((
                    Box::new(Expression::Numeral(Numeral::Integer(2))),
                    BinOp::Pow,
                    Box::new(Expression::BinaryOp((
                        Box::new(Expression::Numeral(Numeral::Integer(3))),
                        BinOp::Pow,
                        Box::new(Expression::Numeral(Numeral::Integer(4)))
                    )))
                ))
            ))
        );

        let input = "(1 + 2)";
        let result = parse_exp(input);

        println!("{:#?}", result);
    }

    #[test]
    fn accepts_unop_exp() {
        let input = "not not true";

        let result = parse_exp(input);

        assert_eq!(
            result,
            Ok((
                "",
                Expression::UnaryOp((
                    UnOp::LogicalNot,
                    Box::new(Expression::UnaryOp((
                        UnOp::LogicalNot,
                        Box::new(Expression::True)
                    )))
                ))
            ))
        );

        let input = "#\"wow!\"";
        let result = parse_exp(input);

        assert_eq!(
            result,
            Ok((
                "",
                Expression::UnaryOp((
                    UnOp::Length,
                    Box::new(Expression::LiteralString(String::from("wow!")))
                ))
            ))
        );
    }

    #[test]
    fn accepts_simple_var() {
        let input = "my_variable;";
        let result = parse_exp(input);

        assert_eq!(
            result,
            Ok((
                ";",
                Expression::PrefixExp(Box::new(PrefixExp::Var(Var::NameVar(String::from(
                    "my_variable"
                )))))
            ))
        );
    }

    #[test]
    fn accepts_functioncall() {
        let input = "launch_missiles( launch_code, 23 )";
        let result = parse_exp(input);

        assert_eq!(
            result,
            Ok((
                "",
                Expression::PrefixExp(Box::new(PrefixExp::FunctionCall(FunctionCall::Standard((
                    Box::new(PrefixExp::Var(Var::NameVar(String::from(
                        "launch_missiles"
                    )))),
                    Args::ExpList(vec![
                        Expression::PrefixExp(Box::new(PrefixExp::Var(Var::NameVar(
                            String::from("launch_code")
                        )))),
                        Expression::Numeral(Numeral::Integer(23))
                    ])
                )))))
            ))
        );
    }

    #[test]
    fn accepts_methodcall() {
        let input = "government:launch_missiles(nil, ...)";
        let result = parse_exp(input);

        assert_eq!(
            result,
            Ok((
                "",
                Expression::PrefixExp(Box::new(PrefixExp::FunctionCall(FunctionCall::Method((
                    Box::new(PrefixExp::Var(Var::NameVar(String::from("government")))),
                    String::from("launch_missiles"),
                    Args::ExpList(vec![Expression::Nil, Expression::DotDotDot])
                )))))
            ))
        )
    }
}
