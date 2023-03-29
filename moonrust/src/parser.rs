pub mod expression;
pub mod statement;
pub mod util;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    multi::{many0, separated_list1},
    sequence::{pair, preceded},
    IResult,
};

// TODO: added lot of `Box`es to avoid infinite recursion, but not sure if this is the best way to do it
use crate::ast::*;
use std::str::FromStr;

use self::{
    expression::parse_dot_dot_dot,
    statement::{parse_return, parse_stmt},
    util::{identifier, ws},
};

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

fn parse_namelist(input: &str) -> IResult<&str, Vec<String>> {
    map(separated_list1(ws(tag(",")), identifier), |result| {
        result.into_iter().map(String::from).collect()
    })(input)
}

fn parse_parlist(input: &str) -> IResult<&str, ParList> {
    alt((
        map(
            pair(
                parse_namelist,
                opt(preceded(ws(char(',')), parse_dot_dot_dot)),
            ),
            |result| ParList(result.0, result.1.is_some()),
        ),
        map(parse_dot_dot_dot, |_| ParList(Vec::new(), true)),
    ))(input)
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
