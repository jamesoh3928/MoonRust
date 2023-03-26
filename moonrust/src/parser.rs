pub mod expression;
pub mod statement;
pub mod util;

use nom::{
    combinator::{map, opt},
    multi::many0,
    sequence::pair,
    IResult,
};

// TODO: added lot of `Box`es to avoid infinite recursion, but not sure if this is the best way to do it
use crate::ast::*;
use std::str::FromStr;

use self::statement::{parse_return, parse_stmt};

/// Just to simplify the return type of each parse function
type ParseResult<'a, T> = IResult<&'a str, T>;

impl FromStr for AST {
    type Err = ASTParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: implement AST parser (may need to create helper functions)
        unimplemented!()
    }
}

/// Parse the input program file into an AST.
pub fn parse(input: &str) -> ParseResult<AST> {
    map(parse_block, |block| AST(block))(input)
}

/// Parse a block. A block is zero or more statements followed by an
/// optional return statement.
fn parse_block(input: &str) -> ParseResult<Block> {
    map(
        pair(many0(parse_stmt), opt(parse_return)),
        |(statements, return_stat)| Block {
            statements,
            return_stat,
        },
    )(input)
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
