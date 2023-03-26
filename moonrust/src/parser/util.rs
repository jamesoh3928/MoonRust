use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, multispace0, one_of},
    combinator::{map_res, opt, recognize},
    error::ParseError,
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated},
    IResult,
};

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
///
/// Credit: https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md#whitespace
pub fn ws<'a, F, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn hexadecimal_value(input: &str) -> IResult<&str, i64> {
    map_res(
        preceded(
            alt((tag("0x"), tag("0X"))),
            recognize(many1(terminated(
                one_of("0123456789abcdefABCDEF"),
                many0(char('_')),
            ))),
        ),
        |out: &str| i64::from_str_radix(&str::replace(&out, "_", ""), 16),
    )(input)
}

// fn decimal(input: &str) -> IResult<&str, i64> {
//     map_res(
//         recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
//         |out: &str| todo!("Fill in the thing"),
//     )(input)
// }

// fn float(input: &str) -> IResult<&str, &str> {
//     alt((
//         // Case one: .42
//         recognize((
//             char('.'),
//             decimal,
//             opt((one_of("eE"), opt(one_of("+-")), decimal)),
//         )), // Case two: 42e42 and 42.42e42
//         recognize((
//             decimal,
//             opt(preceded(char('.'), decimal)),
//             one_of("eE"),
//             opt(one_of("+-")),
//             decimal,
//         )), // Case three: 42. and 42.42
//         recognize((decimal, char('.'), opt(decimal))),
//     ))(input)
// }
