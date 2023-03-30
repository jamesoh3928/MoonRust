use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    multi::{many0, separated_list1},
    sequence::{pair, preceded},
    IResult,
};

use crate::ast::{Block, ParList, Var};

use super::{
    expression::parse_dot_dot_dot,
    statement::{parse_return, parse_stmt},
    util::{identifier, ws},
};

/// Parse a block. A block is zero or more statements followed by an
/// optional return statement.
pub fn parse_block(input: &str) -> IResult<&str, Block> {
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

pub fn parse_parlist(input: &str) -> IResult<&str, ParList> {
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

pub fn parse_var(input: &str) -> IResult<&str, Var> {
    unimplemented!()
}
