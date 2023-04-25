pub mod common;
pub mod expression;
pub mod statement;
pub mod util;
use std::fmt;
use std::fmt::{Display, Formatter};

use nom::{combinator::map, IResult};

// TODO: added lot of `Box`es to avoid infinite recursion, but not sure if this is the best way to do it
use crate::ast::*;
use std::str::FromStr;

use self::common::parse_block;

/// Just to simplify the return type of each parse function
type ParseResult<'a, T> = IResult<&'a str, T>;

impl FromStr for AST {
    type Err = ASTParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse(s) {
            Ok(ast) => Ok(ast.1),
            Err(e) => {
                let msg = e.to_string();
                Err(ASTParseError(format!("Could not parse file: {msg}")))
            }
        }
    }
}

/// Parse the input program file into an AST.
pub fn parse(input: &str) -> ParseResult<AST> {
    map(parse_block, |block| AST(block))(input)
}

#[derive(Debug, PartialEq)]
pub struct ASTParseError(String);
impl Display for ASTParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

// TODO: add unit tests?
// #[cfg(test)]
// mod tests {
//     #[test]
//     fn exploration() {
//         assert_eq!(2 + 2, 4);
//     }

//     #[test]
//     fn another() {
//         panic!("Make this test fail");
//     }
// }
