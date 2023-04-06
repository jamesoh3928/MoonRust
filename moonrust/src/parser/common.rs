use std::result;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    multi::{many0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

use crate::ast::{
    Args, Block, Callee, Expression, Field, FunctionCall, ParList, PrefixExp, Tail, Var,
};

use super::{
    expression::parse_exp,
    statement::{parse_return, parse_stmt},
    util::{identifier, parse_string, ws},
    ParseResult,
};

/// Parse a block. A block is zero or more statements followed by an
/// optional return statement.
pub fn parse_block(input: &str) -> ParseResult<Block> {
    map(
        pair(many0(parse_stmt), opt(parse_return)),
        |(statements, return_stat)| Block {
            statements,
            return_stat,
        },
    )(input)
}

// use for explist!
fn parse_namelist(input: &str) -> ParseResult<Vec<String>> {
    map(separated_list1(ws(tag(",")), identifier), |result| {
        result.into_iter().map(String::from).collect()
    })(input)
}

pub fn parse_parlist(input: &str) -> ParseResult<ParList> {
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

pub fn parse_var(input: &str) -> ParseResult<Var> {
    map(pair(parse_callee, many0(parse_tail)), |result| Var {
        callee: result.0,
        tail: result.1,
    })(input)
}

fn parse_callee(input: &str) -> ParseResult<Callee> {
    alt((
        map(
            delimited(ws(char('(')), parse_exp, ws(char(')'))),
            |result| Callee::WrappedExp(Box::new(result)),
        ),
        map(identifier, |result| Callee::Name(String::from(result))),
    ))(input)
}

fn parse_tail(input: &str) -> ParseResult<Tail> {
    alt((
        map(preceded(char('.'), identifier), |result| {
            Tail::DotIndex(String::from(result))
        }),
        map(
            delimited(ws(char('[')), parse_exp, ws(char(']'))),
            |result| Tail::BracketIndex(result),
        ),
        map(
            pair(
                preceded(char(':'), identifier),
                delimited(
                    char('('),
                    separated_list1(ws(char(',')), parse_exp),
                    char(')'),
                ),
            ),
            |result| Tail::Invoke((String::from(result.0), result.1)),
        ),
        map(
            pair(preceded(char(':'), identifier), parse_table_constructor),
            |result| Tail::InvokeTable((String::from(result.0), result.1)),
        ),
        map(
            pair(preceded(char(':'), identifier), parse_string),
            |result| Tail::InvokeStr((String::from(result.0), result.1)),
        ),
        map(
            delimited(
                char('('),
                separated_list1(ws(char(',')), parse_exp),
                char(')'),
            ),
            |result| Tail::Call(result),
        ),
        map(parse_table_constructor, |result| Tail::Table(result)),
        map(parse_string, |result| Tail::String(result)),
    ))(input)
}

pub fn parse_table_constructor(input: &str) -> ParseResult<Vec<Field>> {
    map(
        delimited(ws(char('{')), opt(parse_fieldlist), ws(char('}'))),
        |result| match result {
            Some(fields) => fields,
            None => Vec::new(),
        },
    )(input)
}

fn parse_fieldlist(input: &str) -> ParseResult<Vec<Field>> {
    separated_list1(ws(alt((char(','), char(';')))), parse_field)(input)
}

fn parse_field(input: &str) -> ParseResult<Field> {
    let result = alt((
        map(
            separated_pair(
                delimited(ws(char('[')), parse_exp, ws(char(']'))),
                ws(char('=')),
                parse_exp,
            ),
            |result| Field::BracketedAssign(result),
        ),
        map(
            separated_pair(ws(identifier), ws(char('=')), ws(parse_exp)),
            |result| Field::NameAssign((String::from(result.0), result.1)),
        ),
        map(parse_exp, |result| Field::UnnamedAssign(result)),
    ))(input);

    result
}

/// prefixexp ::= Name
/// | prefixexp `[` exp `]`
/// | prefixexp `.` Name
/// | prefixexp args
/// | prefixexp `:` Name args
/// | `(` exp `)`
// pub fn parse_prefixexp(input: &str) -> ParseResult<PrefixExp> {
//     alt((
//         map(identifier, |result| {
//             PrefixExp::Var(Var::NameVar(String::from(result)))
//         }),
//         map(
//             pair(
//                 parse_prefixexp,
//                 delimited(ws(char('[')), parse_exp, ws(char(']'))),
//             ),
//             |result| PrefixExp::Var(Var::BracketVar((Box::new(result.0), result.1))),
//         ),
//         map(
//             separated_pair(parse_prefixexp, char('.'), identifier),
//             |result| PrefixExp::Var(Var::DotVar((Box::new(result.0), String::from(result.1)))),
//         ),
//         map(pair(parse_prefixexp, parse_args), |result| {
//             PrefixExp::FunctionCall(FunctionCall::Standard((Box::new(result.0), result.1)))
//         }),
//         map(
//             tuple((
//                 terminated(parse_prefixexp, char(':')),
//                 identifier,
//                 parse_args,
//             )),
//             |result| {
//                 PrefixExp::FunctionCall(FunctionCall::Method((
//                     Box::new(result.0),
//                     String::from(result.1),
//                     result.2,
//                 )))
//             },
//         ),
//         map(
//             delimited(ws(char('(')), parse_exp, ws(char(')'))),
//             |result| PrefixExp::Exp(result),
//         ),
//     ))(input)
//     // alt((
//     //     map(parse_var, |var| PrefixExp::Var(var)),
//     //     map(parse_functioncall, |fncall| PrefixExp::FunctionCall(fncall)),
//     //     map(delimited(ws(char('(')), parse_exp, ws(char(')'))), |exp| {
//     //         PrefixExp::Exp(exp)
//     //     }),
//     // ))(input)
// }

pub fn parse_args(input: &str) -> ParseResult<Args> {
    alt((
        map(separated_list1(ws(char(',')), parse_exp), |result| {
            Args::ExpList(result)
        }),
        map(parse_table_constructor, |result| {
            Args::TableConstructor(result)
        }),
        map(parse_string, |result| Args::LiteralString(result)),
    ))(input)
}

pub fn parse_funcbody(input: &str) -> ParseResult<(ParList, Block)> {
    terminated(
        pair(delimited(char('('), parse_parlist, char(')')), parse_block),
        ws(tag("end")),
    )(input)
}

pub fn parse_dot_dot_dot(input: &str) -> ParseResult<Expression> {
    // DotDotDot, // Used for a variable number of arguments in things like functions
    map(ws(tag("...")), |_| Expression::DotDotDot)(input)
}

pub fn parse_literal_string(input: &str) -> ParseResult<Expression> {
    // TODO(?): I'm ignoring string literals that aren't in double quotes for now
    map(ws(parse_string), |s| Expression::LiteralString(s))(input)
}
