pub mod common;
pub mod expression;
pub mod statement;
pub mod util;
use std::fmt;
use std::fmt::{Display, Formatter};

use nom::{combinator::map, IResult};

use crate::ast::*;
use std::str::FromStr;

use self::common::parse_block;
use self::util::ws;

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
    map(ws(parse_block), |block| AST(block))(input)
}

#[derive(Debug, PartialEq)]
pub struct ASTParseError(String);
impl Display for ASTParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use crate::AST;

    use super::*;

    #[test]
    fn accepts_ast() {
        let input = "
        a = 3 + 5 + 10.0
        a = 3 + 5 + 10.0
        a = 3 + 5 + 10.0
        ";

        let result = parse(input);

        assert_eq!(
            result,
            Ok((
                "",
                AST(Block {
                    statements: vec![
                        Statement::Assignment((
                            vec![Var::NameVar(String::from("a"))],
                            vec![Expression::BinaryOp((
                                Box::new(Expression::BinaryOp((
                                    Box::new(Expression::Numeral(Numeral::Integer(3))),
                                    BinOp::Add,
                                    Box::new(Expression::Numeral(Numeral::Integer(5)))
                                ))),
                                BinOp::Add,
                                Box::new(Expression::Numeral(Numeral::Float(10.0)))
                            ))],
                            false
                        )),
                        Statement::Assignment((
                            vec![Var::NameVar(String::from("a"))],
                            vec![Expression::BinaryOp((
                                Box::new(Expression::BinaryOp((
                                    Box::new(Expression::Numeral(Numeral::Integer(3))),
                                    BinOp::Add,
                                    Box::new(Expression::Numeral(Numeral::Integer(5)))
                                ))),
                                BinOp::Add,
                                Box::new(Expression::Numeral(Numeral::Float(10.0)))
                            ))],
                            false
                        )),
                        Statement::Assignment((
                            vec![Var::NameVar(String::from("a"))],
                            vec![Expression::BinaryOp((
                                Box::new(Expression::BinaryOp((
                                    Box::new(Expression::Numeral(Numeral::Integer(3))),
                                    BinOp::Add,
                                    Box::new(Expression::Numeral(Numeral::Integer(5)))
                                ))),
                                BinOp::Add,
                                Box::new(Expression::Numeral(Numeral::Float(10.0)))
                            ))],
                            false
                        ))
                    ],
                    return_stat: None
                })
            ))
        )
    }
}
