use nom::character::complete::char;
use nom::combinator::{opt, value};
use nom::multi::{many0, separated_list1};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{pair, preceded, tuple},
};

use super::common::{parse_args, parse_funcbody, parse_prefixexp, parse_table_constructor};
use super::{util::*, ParseResult};

use crate::ast::{Args, Expression, FunctionCall, Statement};
use crate::parser::common::{parse_block, parse_parlist, parse_var};
use crate::parser::expression;

pub fn parse_stmt(input: &str) -> ParseResult<Statement> {
    alt((
        //parse_semicolon,
        parse_assignment,
        parse_functioncall_statement,
        parse_break,
        parse_while,
        parse_repeat,
        parse_do_block,
        parse_if,
        parse_for_num,
        parse_for_generic,
        parse_function_decl,
        local_func_decl,
    ))(input)
}
/// Parse a single semicolon. Toss the result since it provides no
/// semantic information.
fn parse_semicolon(input: &str) -> ParseResult<()> {
    value((), char(';'))(input)
}

fn parse_assignment(input: &str) -> ParseResult<Statement> {
    //Assignment((Vec<Var>, Vec<Expression>))

    map(
        tuple((
            separated_list1(ws(alt((char(','), char(';')))), parse_var),
            separated_list1(ws(alt((char(','), char(';')))), expression::parse_exp),
        )),
        |result| Statement::Assignment(result),
    )(input)
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

pub fn parse_return(input: &str) -> ParseResult<Vec<Expression>> {
    unimplemented!()
}
