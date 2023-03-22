// TODO: added lot of `Box`es to avoid infinite recursion, but not sure if this is the best way to do it
use crate::ast::*;
use std::str::FromStr;

impl FromStr for AST {
    type Err = ASTParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: implement AST parser (may need to create helper functions)
        unimplemented!()
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
