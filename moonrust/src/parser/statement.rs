use nom::character::complete::char;
use nom::combinator::value;
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{delimited, pair, preceded, terminated, tuple},
};

use super::{
    util::*,
    ParseResult,
};

use crate::parser::expression;

use crate::ast::{Expression, FunctionCall, Statement};

/// Parse a single semicolon. Toss the result since it provides no
/// semantic information.
fn parse_semicolon(input: &str) -> ParseResult<()> {
    value((), char(';'))(input)
}

fn parse_assignment(input: &str) -> ParseResult<Statement> {
    //Assignment((Vec<Var>, Vec<Expression>))
    unimplemented!()
}

pub fn parse_functioncall(input: &str) -> ParseResult<FunctionCall> {
    // FunctionCall((PrefixExp, Option<String>))
    unimplemented!()
}

fn parse_break(input: &str) -> ParseResult<Statement> {
    map(ws(tag("break")), |_| Statement::Break)(input)
}

fn parse_do_block(input: &str) ->ParseResult<Statement> {
    // DoBlock(Block)
    unimplemented!()
}

fn parse_while(input: &str) -> ParseResult<Statement> {
    // While((Expression, Block))
    unimplemented!()
}

fn parse_repeat(input: &str) -> ParseResult<Statement> {
    // Repeat((Block, Expression))
    unimplemented!()
}

fn parse_if(input: &str) -> ParseResult<Statement> {
    // If((Expression, Block, Vec<(Expression, Block)>, Option<Block>))
    unimplemented!()
}

fn parse_for_num(input: &str) -> ParseResult<Statement> {
    // ForNum((String, i64, i64, Option<i64>))
    unimplemented!()
}

fn parse_for_generic(input: &str) -> ParseResult<Statement> {
    // ForGeneric((Vec<String>, Vec<Expression>, Block))
    unimplemented!()
}

fn parse_function_decl(input: &str) -> ParseResult<Statement> {
    // FunctionDecl((String, ParList, Block))
    map( tuple( (ws(parse_string), preceded(ws(tag("function")), expression::parse_funcbody)) ),  
    |result| Statement::FunctionDecl(result) )(input)

}

fn local_func_decl(input: &str) -> ParseResult<Statement> {
    // LocalFuncDecl((String, ParList, Block))
    unimplemented!()
}

pub fn parse_stmt(input: &str) -> ParseResult<Statement> {
    unimplemented!()
}

pub fn parse_return(input: &str) -> ParseResult<Vec<Expression>> {
    unimplemented!()
}






