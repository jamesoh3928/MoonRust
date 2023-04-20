use nom::character::complete::char;
use nom::combinator::{fail, opt, verify};
use nom::multi::{many0, separated_list1};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{pair, preceded, tuple},
};

use super::common::{parse_args, parse_funcbody, parse_prefixexp};
use super::{util::*, ParseResult};

use crate::ast::{Expression, FunctionCall, PrefixExp, Statement};
use crate::parser::common::{parse_block, parse_parlist};
use crate::parser::expression;

pub fn parse_stmt(input: &str) -> ParseResult<Statement> {
    alt((
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
        local_func_decl,
        // parse_local_assgn,
    ))(input)
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
    map(parse_block, |block| Statement::DoBlock(block))(input)
}

fn parse_while(input: &str) -> ParseResult<Statement> {
    // While((Expression, Block))
    map(
        tuple((ws(tag("while")), expression::parse_exp, parse_block)),
        |result| Statement::While((result.1, result.2)),
    )(input)
}

fn parse_repeat(input: &str) -> ParseResult<Statement> {
    // Repeat((Block, Expression))
    map(
        tuple((ws(tag("repeat")), parse_block, expression::parse_exp)),
        |result| Statement::Repeat((result.1, result.2)),
    )(input)
}

fn parse_if(input: &str) -> ParseResult<Statement> {
    // If((Expression, Block, Vec<(Expression, Block)>, Option<Block>))
    map(
        tuple((
            ws(tag("if")),
            expression::parse_exp,
            ws(tag("then")),
            parse_block,
            many0(tuple((
                preceded(ws(tag("elseif")), expression::parse_exp),
                preceded(ws(tag("then")), parse_block),
            ))),
            ws(tag("else")),
            opt(parse_block),
            ws(tag("end")),
        )),
        |result| Statement::If((result.1, result.3, result.4, result.6)),
    )(input)
}

fn parse_for_num(input: &str) -> ParseResult<Statement> {
    // ForNum((String, Expression, Expression, Option<Expression>, Block))

    map(
        tuple((
            pair(
                ws(tag("for")),
                tuple((
                    expression::parse_exp,
                    expression::parse_exp,
                    opt(expression::parse_exp),
                )),
            ),
            parse_block,
        )),
        |result| {
            Statement::ForNum((
                String::from(result.0 .0),
                result.0 .1 .0,
                result.0 .1 .1,
                result.0 .1 .2,
                result.1,
            ))
        },
    )(input)
}

// redo
fn parse_for_generic(input: &str) -> ParseResult<Statement> {
    // ForGeneric((Vec<String>, Vec<Expression>, Block))

    map(
        tuple((
            ws(tag("for")),
            separated_list1(ws(alt((char(','), char(';')))), parse_string),
            separated_list1(ws(alt((char(','), char(';')))), expression::parse_exp),
            preceded(parse_parlist, parse_block),
        )),
        |result| Statement::ForGeneric((result.1, result.2, result.3)),
    )(input)
}

fn parse_function_decl(input: &str) -> ParseResult<Statement> {
    // FunctionDecl((String, ParList, Block)) where String = name of function being declared
    map(
        tuple((
            ws(tag("function")),
            ws(identifier),
            preceded(parse_parlist, parse_funcbody),
        )),
        |result| Statement::FunctionDecl((String::from(result.1), result.2 .0, result.2 .1)),
    )(input)
}

fn local_func_decl(input: &str) -> ParseResult<Statement> {
    // LocalFuncDecl((String, ParList, Block))
    map(
        tuple((
            ws(tag("function")),
            ws(identifier),
            preceded(parse_parlist, parse_funcbody),
        )),
        |result| Statement::LocalFuncDecl((String::from(result.1), result.2 .0, result.2 .1)),
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
                            verify(parse_prefixexp, |pexp| match pexp {
                                PrefixExp::Var(_) => true,
                                _ => false,
                            }),
                            |result| {
                                println!("{:?}", result);
                                match result {
                                    PrefixExp::Var(var) => var,
                                    _ => unreachable!(),
                                }
                            },
                        ),
                    ),
                    opt(preceded(
                        ws(char('=')),
                        separated_list1(ws(char(',')), expression::parse_exp),
                    )),
                ),
                |result| {
                    let success = !(result.1.is_none() && !is_local);
                    println!("{success}");
                    success
                },
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
    unimplemented!()
}

#[cfg(test)]
mod tests {

    use crate::ast::{Args, BinOp, Block, Numeral, PrefixExp, Var};

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
            "  ",
            Statement::Assignment((
                vec![
                    Var::NameVar(String::from("r")),
                    Var::NameVar(String::from("v")),
                ],
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

        let input = "and(true, false)";
        let expected = Ok((
            "",
            Statement::FunctionCall(FunctionCall::Standard((
                Box::new(PrefixExp::Var(Var::NameVar(String::from("and")))),
                Args::ExpList(vec![Expression::True, Expression::False]),
            ))),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_functioncall_statement() {}

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
                statements: vec![Statement::Assignment((
                    vec![
                        Var::NameVar(String::from("a")),
                        Var::NameVar(String::from("b")),
                    ],
                    vec![
                        Expression::BinaryOp((
                            Box::new(Expression::LiteralString(String::from("a"))),
                            BinOp::Equal,
                            Box::new(Expression::Numeral(Numeral::Integer(1))),
                        )),
                        Expression::BinaryOp((
                            Box::new(Expression::LiteralString(String::from("b"))),
                            BinOp::Equal,
                            Box::new(
                                Expression::Numeral(Numeral::Integer(3)), // a + 3?
                                                                          // (Expression::LiteralString(String::from("a")),
                                                                          // BinOp::Add,
                                                                          // Expression::Numeral(
                                                                          //     Numeral::Integer(3)
                                                                          // ))
                            ),
                        )),
                    ],
                    true,
                ))],
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
                    Box::new(Expression::LiteralString(String::from("i"))),
                    BinOp::LessEq,
                    Box::new(Expression::LiteralString(String::from("x"))),
                )),
                Block {
                    statements: vec![Statement::Assignment((
                        vec![Var::NameVar(String::from("x"))],
                        vec![Expression::BinaryOp((
                            Box::new(Expression::LiteralString(String::from("i"))),
                            BinOp::Mult,
                            Box::new(Expression::Numeral(Numeral::Integer(2))),
                        ))],
                        true,
                    ))],
                    return_stat: None,
                },
            )),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn accepts_repeat() {
        // repeat block until exp
        // Repeat((Block, Expression))

        let input = "
            a = 10
            repeat
                a = a + 1
                until(a > 15)
        ";

        let expected = Ok((
            "",
            // Statement::Assignment((
            //     vec![
            //       Var::NameVar(
            //           String::from("a"),
            //       ),
            //     ],
            //     vec![
            //         Expression::BinaryOp((
            //             Box::new(
            //                 Expression::LiteralString(String::from("a"))
            //             ),
            //             BinOp::Equal,
            //             Box::new(
            //                 Expression::Numeral(
            //                     Numeral::Integer(10)
            //                 )
            //             )
            //         )),
            //     ]
            //   )),
            Statement::Repeat((
                Block {
                    statements: vec![Statement::Assignment((
                        vec![Var::NameVar(String::from("a"))],
                        vec![Expression::BinaryOp((
                            Box::new(Expression::LiteralString(String::from("a"))),
                            BinOp::Add,
                            Box::new(Expression::Numeral(Numeral::Integer(1))),
                        ))],
                        false,
                    ))],
                    return_stat: None,
                },
                Expression::BinaryOp((
                    Box::new(Expression::LiteralString(String::from("a"))),
                    BinOp::GreaterThan,
                    Box::new(Expression::Numeral(Numeral::Integer(15))),
                )),
            )),
        ));

        let actual = parse_stmt(input);
        assert_eq!(expected, actual);
    }

    // #[test]
    // fn accepts_if() {
    //     // If((Expression, Block, Vec<(Expression, Block)>, Option<Block>))
    //         //print statement = function call

    //     let input =
    //     "
    //         if(c < 43)
    //         then
    //             print(c is less than 43)
    //         elseif (c > 43)
    //         then
    //             print(c is greater than 43)
    //         else
    //             print(c is equal to 43)
    //         end

    //         print(the value of c is: , c)
    //     ";

    //     let expected = Ok((
    //         "", // unconsumed data
    //         Statement::If((
    //             // if
    //             Expression::BinaryOp((
    //                 Box::new(
    //                     Expression::PrefixExp(
    //     Box::new(
    //         PrefixExp::Var(Var::NameVar(String::from("i")))
    //     )
    // )

    //                 ),
    //                 BinOp::LessThan,
    //                 Box::new(
    //                     Expression::Numeral(
    //                         Numeral::Integer(43)
    //                     )
    //                 )
    //             )),
    //             Block {
    //                 statements: vec![
    //                     Statement::Assignment((
    //                         vec![
    //                             Var::NameVar(
    //                                 String::from("c"),
    //                             )
    //                         ],
    //                         vec![
    //                             Expression::BinaryOp((
    //                                 Box::new(
    //                                     Expression::Nil
    //                                 ),
    //                                 BinOp::GreaterThan,
    //                                 Box::new(
    //                                     Expression::Numeral(
    //                                         Numeral::Integer(43)
    //                                     )
    //                                 )

    //                             ))
    //                         ]
    //                     ))
    //                 ],
    //                 return_stat: None,
    //             },
    //             // elseif
    //             vec![(
    //                 Expression::LiteralString(String::from("c")),
    //                 Block {

    //                 },
    //             )],
    //             // else
    //             None

    //         ))

    //     ));

    //     let actual = parse_stmt(input);
    //     assert_eq!(expected, actual);

    // }

    #[test]
    fn accepts_for_num() {
        // ForNum((String, Expression, Expression, Option<Expression>, Block))
    }
}
