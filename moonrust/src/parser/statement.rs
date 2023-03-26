use nom::character::complete::char;
use nom::combinator::value;

use crate::ast::{Expression, Statement};

use super::ParseResult;

pub fn parse_stmt(input: &str) -> ParseResult<Statement> {
    unimplemented!()
}

pub fn parse_return(input: &str) -> ParseResult<Vec<Expression>> {
    unimplemented!()
}

/// Parse a single semicolon. Toss the result since it provides no
/// semantic information.
fn parse_semicolon(input: &str) -> ParseResult<()> {
    value((), char(';'))(input)
}
