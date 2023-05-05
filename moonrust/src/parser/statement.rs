use nom::character::complete::char;
use nom::combinator::{complete, fail, opt, verify};
use nom::multi::{many0, separated_list0, separated_list1};
use nom::sequence::{delimited, terminated};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{pair, preceded, tuple},
};

use super::common::{parse_args, parse_funcbody, parse_prefixexp};
use super::expression::parse_exp;
use super::{util::*, ParseResult};

use crate::ast::{Expression, FunctionCall, PrefixExp, Statement};
use crate::parser::common::parse_block;
use crate::parser::expression;

pub fn parse_stmt(input: &str) -> ParseResult<Statement> {
    complete(alt((
        parse_semicolon,
        parse_stmt_prefixexp,
        parse_break,
        parse_while,
        parse_repeat,
        parse_do_block,
        parse_if,
        parse_for_num,
        parse_for_generic,
        parse_function_decl,
        parse_local_func_decl,
    )))(input)
}
/// Parse a single semicolon. Toss the result since it provides no
/// semantic information.
fn parse_semicolon(input: &str) -> ParseResult<Statement> {
    map(ws(tag(";")), |_| Statement::Semicolon)(input)
}

pub fn parse_functioncall(input: &str) -> ParseResult<FunctionCall> {
    // FunctionCall((PrefixExp, Option<String>))

    alt((
        map(tuple((ws(parse_prefixexp), ws(parse_args))), |result| {
            FunctionCall::Standard((Box::new(result.0), result.1))
        }),
        map(
            tuple((
                ws(parse_prefixexp),
                ws(char(':')),
                ws(identifier),
                ws(parse_args),
            )),
            |result| FunctionCall::Method((Box::new(result.0), String::from(result.2), result.3)),
        ),
    ))(input)
}

pub fn parse_functioncall_statement(input: &str) -> ParseResult<Statement> {
    // FunctionCall((PrefixExp, Option<String>))
    map(tuple((parse_functioncall, opt(parse_string))), |result| {
        Statement::FunctionCall(result.0)
    })(input)
}

fn parse_break(input: &str) -> ParseResult<Statement> {
    map(ws(tag("break")), |_| Statement::Break)(input)
}

fn parse_do_block(input: &str) -> ParseResult<Statement> {
    // DoBlock(Block)
    map(
        delimited(ws(tag("do")), parse_block, ws(tag("end"))),
        Statement::DoBlock,
    )(input)
}

fn parse_while(input: &str) -> ParseResult<Statement> {
    // While((Expression, Block))
    map(
        tuple((
            ws(tag("while")),
            expression::parse_exp,
            delimited(ws(tag("do")), parse_block, ws(tag("end"))),
        )),
        |result| Statement::While((result.1, result.2)),
    )(input)
}

fn parse_repeat(input: &str) -> ParseResult<Statement> {
    // Repeat((Block, Expression))
    map(
        pair(
            preceded(ws(tag("repeat")), parse_block),
            preceded(ws(tag("until")), expression::parse_exp),
        ),
        Statement::Repeat,
    )(input)
}

fn parse_if(input: &str) -> ParseResult<Statement> {
    // If((Expression, Block, Vec<(Expression, Block)>, Option<Block>))
    map(
        tuple((
            preceded(ws(tag("if")), parse_exp),
            preceded(ws(tag("then")), parse_block),
            many0(pair(
                preceded(ws(tag("elseif")), parse_exp),
                preceded(ws(tag("then")), parse_block),
            )),
            terminated(opt(preceded(ws(tag("else")), parse_block)), ws(tag("end"))),
        )),
        Statement::If,
    )(input)
}

fn parse_for_num(input: &str) -> ParseResult<Statement> {
    // ForNum((String, Expression, Expression, Option<Expression>, Block))

    map(
        tuple((
            map(preceded(ws(tag("for")), identifier), String::from),
            preceded(ws(tag("=")), parse_exp),
            preceded(ws(tag(",")), parse_exp),
            opt(preceded(ws(tag(",")), parse_exp)),
            delimited(ws(tag("do")), parse_block, ws(tag("end"))),
        )),
        Statement::ForNum,
    )(input)
}

// redo
fn parse_for_generic(input: &str) -> ParseResult<Statement> {
    // ForGeneric((Vec<String>, Vec<Expression>, Block))
    map(
        tuple((
            preceded(
                ws(tag("for")),
                separated_list1(ws(tag(",")), map(identifier, String::from)),
            ),
            preceded(ws(tag("in")), separated_list1(ws(tag(",")), parse_exp)),
            delimited(ws(tag("do")), parse_block, ws(tag("end"))),
        )),
        Statement::ForGeneric,
    )(input)
}

fn parse_function_decl(input: &str) -> ParseResult<Statement> {
    // FunctionDecl((String, ParList, Block)) where String = name of function being declared
    map(
        pair(
            preceded(
                ws(tag("function")),
                map(
                    pair(
                        separated_list1(char('.'), identifier),
                        opt(preceded(ws(char(':')), identifier)),
                    ),
                    |result| {
                        let mut name = String::new();
                        for st in result.0.into_iter() {
                            name.push_str(st);
                        }
                        name.push_str(result.1.unwrap_or(""));
                        name
                    },
                ),
            ),
            parse_funcbody,
        ),
        |result| Statement::FunctionDecl((result.0, result.1 .0, result.1 .1)),
    )(input)
}

fn parse_local_func_decl(input: &str) -> ParseResult<Statement> {
    // LocalFuncDecl((String, ParList, Block))
    map(
        preceded(
            pair(ws(tag("local")), ws(tag("function"))),
            pair(map(identifier, String::from), parse_funcbody),
        ),
        |result| Statement::LocalFuncDecl((result.0, result.1 .0, result.1 .1)),
    )(input)
}

fn parse_stmt_prefixexp(input: &str) -> ParseResult<Statement> {
    let (input_after_local, is_local) =
        map(opt(ws(tag("local"))), |result| result.is_some())(input)?;
    let (rest_input, pexp) = parse_prefixexp(input_after_local)?;

    if let PrefixExp::FunctionCall(fncall) = pexp {
        if is_local {
            fail("Function calls cannot be local")
        } else {
            Ok((rest_input, Statement::FunctionCall(fncall)))
        }
    } else {
        map(
            verify(
                pair(
                    separated_list1(
                        ws(char(',')),
                        map(
                            verify(parse_prefixexp, |pexp| matches!(pexp, PrefixExp::Var(_))),
                            |result| match result {
                                PrefixExp::Var(var) => var,
                                _ => unreachable!(),
                            },
                        ),
                    ),
                    opt(preceded(
                        ws(char('=')),
                        separated_list1(ws(char(',')), expression::parse_exp),
                    )),
                ),
                |result| result.1.is_some() || is_local,
            ),
            |result| match result.1 {
                Some(exps) => Statement::Assignment((result.0, exps, is_local)),
                None => {
                    let vars_len = result.0.len();
                    let mut exp_vec = Vec::new();
                    for _ in 0..vars_len {
                        exp_vec.push(Expression::Nil);
                    }
                    Statement::Assignment((result.0, exp_vec, is_local))
                }
            },
        )(input_after_local)
    }
}

// used in parse_block, not considered a Lua statement
pub fn parse_return(input: &str) -> ParseResult<Vec<Expression>> {
    // retstat ::= return [explist] [‘;’]
    // explist and ; are optional
    preceded(
        ws(tag("return")),
        terminated(
            separated_list0(ws(char(',')), parse_exp),
            opt(ws(char(';'))),
        ),
    )(input)
}

#[cfg(test)]
mod tests {

    use crate::ast::{Args, BinOp, Block, Numeral, ParList, PrefixExp, UnOp, Var};

    use super::*;

    #[test]
    fn accepts_semicolon() {
        let expected = parse_semicolon(";");
        assert_eq!(expected, Ok(("", Statement::Semicolon)));

        let expected = parse_stmt("     ;     ");
        assert_eq!(expected, Ok(("", Statement::Semicolon)));
    }

    #[test]
    fn accepts_assignment() {
        // Assignment((Vec<Var>, Vec<Expression>))
        let input = "local   r,v  ";

        let expected = Ok((
            "",
            Statement::Assignment((
                vec![Var::Name(String::from("r")), Var::Name(String::from("v"))],
                vec![Expression::Nil, Expression::Nil],
                true,
            )),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_functioncall() {
        // FunctionCall(FunctionCall)
        // functioncall ::=  prefixexp args | prefixexp ‘:’ Name args

        let input = "do_thing(true, false)";
        let expected = Ok((
            "",
            Statement::FunctionCall(FunctionCall::Standard((
                Box::new(PrefixExp::Var(Var::Name(String::from("do_thing")))),
                Args::ExpList(vec![Expression::True, Expression::False]),
            ))),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_break() {
        let expected = parse_stmt("break");
        assert_eq!(expected, Ok(("", Statement::Break)));

        let expected = parse_stmt("     break     ");
        assert_eq!(expected, Ok(("", Statement::Break)));
    }

    #[test]
    fn accepts_do_block() {
        // DoBlock(Block)
        let input = "do 
            local a = 1
            b = a + 3
        end";

        let expected = Ok((
            "",
            Statement::DoBlock(Block {
                statements: vec![
                    Statement::Assignment((
                        vec![Var::Name(String::from("a"))],
                        vec![Expression::Numeral(Numeral::Integer(1))],
                        true,
                    )),
                    Statement::Assignment((
                        vec![Var::Name(String::from("b"))],
                        vec![Expression::BinaryOp((
                            Box::new(Expression::PrefixExp(Box::new(PrefixExp::Var(Var::Name(
                                String::from("a"),
                            ))))),
                            BinOp::Add,
                            Box::new(Expression::Numeral(Numeral::Integer(3))),
                        ))],
                        false,
                    )),
                ],
                return_stat: None,
            }),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_while() {
        // While((Expression, Block))

        let input = "while i <= x do
            local x = i*2
            print(x)
            i = i + 1
        end
        ";

        let expected = Ok((
            "",
            Statement::While((
                Expression::BinaryOp((
                    Box::new(Expression::PrefixExp(Box::new(PrefixExp::Var(Var::Name(
                        String::from("i"),
                    ))))),
                    BinOp::LessEq,
                    Box::new(Expression::PrefixExp(Box::new(PrefixExp::Var(Var::Name(
                        String::from("x"),
                    ))))),
                )),
                Block {
                    statements: vec![
                        Statement::Assignment((
                            vec![Var::Name(String::from("x"))],
                            vec![Expression::BinaryOp((
                                Box::new(Expression::PrefixExp(Box::new(PrefixExp::Var(
                                    Var::Name(String::from("i")),
                                )))),
                                BinOp::Mult,
                                Box::new(Expression::Numeral(Numeral::Integer(2))),
                            ))],
                            true,
                        )),
                        Statement::FunctionCall(FunctionCall::Standard((
                            Box::new(PrefixExp::Var(Var::Name(String::from("print")))),
                            Args::ExpList(vec![Expression::PrefixExp(Box::new(PrefixExp::Var(
                                Var::Name(String::from("x")),
                            )))]),
                        ))),
                        Statement::Assignment((
                            vec![Var::Name(String::from("i"))],
                            vec![Expression::BinaryOp((
                                Box::new(Expression::PrefixExp(Box::new(PrefixExp::Var(
                                    Var::Name(String::from("i")),
                                )))),
                                BinOp::Add,
                                Box::new(Expression::Numeral(Numeral::Integer(1))),
                            ))],
                            false,
                        )),
                    ],
                    return_stat: None,
                },
            )),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_repeat() {
        // Repeat((Block, Expression))
        // repeat block until exp

        let input = "
            repeat
                a = a + 1
                until(a > 15)
        ";

        let expected = Ok((
            "", // uncomsumed data
            Statement::Repeat((
                Block {
                    statements: vec![Statement::Assignment((
                        vec![Var::Name(String::from("a"))],
                        vec![Expression::BinaryOp((
                            Box::new(Expression::PrefixExp(Box::new(PrefixExp::Var(Var::Name(
                                String::from("a"),
                            ))))),
                            BinOp::Add,
                            Box::new(Expression::Numeral(Numeral::Integer(1))),
                        ))],
                        false,
                    ))],
                    return_stat: None,
                },
                Expression::PrefixExp(Box::new(PrefixExp::Exp(Expression::BinaryOp((
                    Box::new(Expression::PrefixExp(Box::new(PrefixExp::Var(Var::Name(
                        String::from("a"),
                    ))))),
                    BinOp::GreaterThan,
                    Box::new(Expression::Numeral(Numeral::Integer(15))),
                ))))),
            )),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_if() {
        // If((Expression, Block, Vec<(Expression, Block)>, Option<Block>))
        //print statement = function call

        let input = "
            if(c < 43)
            then
                return \"yes\"
            elseif (c > 43)
            then
                return \"no\"
            else
                return \"maybe\"
            end
        ";

        let expected = Ok((
            "",
            Statement::If((
                Expression::PrefixExp(Box::new(PrefixExp::Exp(Expression::BinaryOp((
                    Box::new(Expression::PrefixExp(Box::new(PrefixExp::Var(Var::Name(
                        String::from("c"),
                    ))))),
                    BinOp::LessThan,
                    Box::new(Expression::Numeral(Numeral::Integer(43))),
                ))))),
                Block {
                    statements: vec![],
                    return_stat: Some(vec![Expression::LiteralString(String::from("yes"))]),
                },
                vec![(
                    Expression::PrefixExp(Box::new(PrefixExp::Exp(Expression::BinaryOp((
                        Box::new(Expression::PrefixExp(Box::new(PrefixExp::Var(Var::Name(
                            String::from("c"),
                        ))))),
                        BinOp::GreaterThan,
                        Box::new(Expression::Numeral(Numeral::Integer(43))),
                    ))))),
                    Block {
                        statements: vec![],
                        return_stat: Some(vec![Expression::LiteralString(String::from("no"))]),
                    },
                )],
                Some(Block {
                    statements: vec![],
                    return_stat: Some(vec![Expression::LiteralString(String::from("maybe"))]),
                }),
            )),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_for_num() {
        // ForNum((String, Expression, Expression, Option<Expression>, Block))
        let input = "
            for i = 10,1,-1
            do
                print(i)
            end
        ";

        let expected = Ok((
            "",
            Statement::ForNum((
                String::from("i"),
                Expression::Numeral(Numeral::Integer(10)),
                Expression::Numeral(Numeral::Integer(1)),
                Some(Expression::UnaryOp((
                    UnOp::Negate,
                    Box::new(Expression::Numeral(Numeral::Integer(1))),
                ))),
                Block {
                    statements: vec![Statement::FunctionCall(FunctionCall::Standard((
                        Box::new(PrefixExp::Var(Var::Name(String::from("print")))),
                        Args::ExpList(vec![Expression::PrefixExp(Box::new(PrefixExp::Var(
                            Var::Name(String::from("i")),
                        )))]),
                    )))],
                    return_stat: None,
                },
            )),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_for_generic() {
        // ForGeneric((Vec<String>, Vec<Expression>, Block))

        let input = "
            for index, name in names do
                print(name)
                print(index)
            end
        ";

        let expected = Ok((
            "",
            Statement::ForGeneric((
                vec![String::from("index"), String::from("name")],
                vec![Expression::PrefixExp(Box::new(PrefixExp::Var(Var::Name(
                    String::from("names"),
                ))))],
                Block {
                    statements: vec![
                        Statement::FunctionCall(FunctionCall::Standard((
                            Box::new(PrefixExp::Var(Var::Name(String::from("print")))),
                            Args::ExpList(vec![Expression::PrefixExp(Box::new(PrefixExp::Var(
                                Var::Name(String::from("name")),
                            )))]),
                        ))),
                        Statement::FunctionCall(FunctionCall::Standard((
                            Box::new(PrefixExp::Var(Var::Name(String::from("print")))),
                            Args::ExpList(vec![Expression::PrefixExp(Box::new(PrefixExp::Var(
                                Var::Name(String::from("index")),
                            )))]),
                        ))),
                    ],
                    return_stat: None,
                },
            )),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_function_decl() {
        // FunctionDecl((String, ParList, Block))

        let input = "
            function num(num1)

                return num1; 
            end
        ";

        let expected = Ok((
            "",
            Statement::FunctionDecl((
                String::from("num"),
                ParList(vec![String::from("num1")], false),
                Block {
                    statements: vec![],
                    return_stat: Some(vec![Expression::PrefixExp(Box::new(PrefixExp::Var(
                        Var::Name(String::from("num1")),
                    )))]),
                },
            )),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_local_func_def() {
        // LocalFuncDecl((String, ParList, Block))

        let input = "
            local function fact(n)
                if n == 0 then return 1
                end
            end 
        ";

        let expected = Ok((
            "",
            Statement::LocalFuncDecl((
                String::from("fact"),
                ParList(vec![String::from("n")], false),
                Block {
                    statements: vec![Statement::If((
                        Expression::BinaryOp((
                            Box::new(Expression::PrefixExp(Box::new(PrefixExp::Var(Var::Name(
                                String::from("n"),
                            ))))),
                            BinOp::Equal,
                            Box::new(Expression::Numeral(Numeral::Integer(0))),
                        )),
                        Block {
                            statements: vec![],
                            return_stat: Some(vec![Expression::Numeral(Numeral::Integer(1))]),
                        },
                        vec![],
                        None,
                    ))],
                    return_stat: None,
                },
            )),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }
}
