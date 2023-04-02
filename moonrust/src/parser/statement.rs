use nom::character::complete::char;
use nom::combinator::value;
use nom::multi::{many1, separated_list1};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    sequence::{delimited, pair, preceded, terminated, tuple},
};

use super::expression::parse_table_constructor;
use super::{
    util::*,
    ParseResult,
};

use crate::parser::expression;
use crate::parser::common;
use crate::ast::{Expression, FunctionCall, Statement, Args};

pub fn parse_stmt(input: &str) -> ParseResult<Statement> {

    alt((
        //parse_semicolon,
        parse_assignment,
        parse_function_decl,
        parse_break,
        parse_do_block,
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
    unimplemented!()
}

fn parse_args(input: &str) -> ParseResult<Args> {

    alt((
        map( separated_list1(ws(char(',')), expression::parse_exp), |result| Args::ExpList(result) ),
        map( parse_table_constructor, |result| Args::TableConstructor(result)),
        map( parse_string, |result|  Args::LiteralString(result)),
    ))(input)
}
pub fn functioncall(input: &str) -> ParseResult<FunctionCall> {
    // FunctionCall((PrefixExp, Option<String>))

    alt(( 
        map( tuple( (ws(expression::parse_prefixexp), ws(parse_args)) ),  |result| FunctionCall::Standard((Box::new(result.0), result.1))),
        map( tuple( (ws(expression::parse_prefixexp), ws(char(':')), ws(identifier), ws(parse_args)) ),  |result| FunctionCall::Method((Box::new(result.0), String::from(result.2), result.3))),

    ))(input)

}

pub fn parse_functioncall_statement(input: &str) -> ParseResult<Statement> {
    // FunctionCall((PrefixExp, Option<String>))
    map( functioncall, |result| Statement::FunctionCall(result) )(input)
}

fn parse_break(input: &str) -> ParseResult<Statement> {
    map(ws(tag("break")), |_| Statement::Break)(input)
}

fn parse_do_block(input: &str) ->ParseResult<Statement> {
    // DoBlock(Block)
    map(expression::parse_funcbody, |block| Statement::DoBlock(block.1))(input)
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
    // ForNum((String, Expression, Expression, Option<Expression>, Block))
    /* NOT SURE HOW TO DO OPTION<EXPRESSION> */
    map( tuple( (preceded(ws(tag("for")), tuple( (expression::parse_exp, expression::parse_exp, common::parse_block) ))) ), |result| Statement::ForNum(result) )(input)

}

fn parse_for_generic(input: &str) -> ParseResult<Statement> {
    // ForGeneric((Vec<String>, Vec<Expression>, Block))
    unimplemented!()
}

fn parse_function_decl(input: &str) -> ParseResult<Statement> {
    // FunctionDecl((String, ParList, Block)) where String = name of function being declared
    map( tuple( (ws(tag("function")), ws(identifier), preceded(common::parse_parlist, expression::parse_funcbody)) ),  
    |result| Statement::FunctionDecl( (String::from(result.1), result.2.0, result.2.1)) )(input)

}

fn local_func_decl(input: &str) -> ParseResult<Statement> {
    // LocalFuncDecl((String, ParList, Block))
    map( tuple( (ws(tag("function")), ws(identifier), preceded(common::parse_parlist, expression::parse_funcbody)) ),  
    |result| Statement::LocalFuncDecl( (String::from(result.1), result.2.0, result.2.1)) )(input)

}

pub fn parse_return(input: &str) -> ParseResult<Vec<Expression>> {
    unimplemented!()
}

