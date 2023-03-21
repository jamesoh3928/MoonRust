// TODO: added lot of `Box`es to avoid infinite recursion, but not sure if this is the best way to do it
enum Statement {
    Semicolon,
    Assignment((Vec<Var>, Vec<Expression>)),
    FunctionCall((PrefixExp, Option<String>)),
    Break,
    DoBlock(Block),
    While((Expression, Block)),
    Repeat((Block, Expression)),
    If((Expression, Block, Vec<(Expression, Block)>, Option<Block>)),
    ForNum((String, i64, i64, Option<i64>)),
    ForGeneric((Vec<String>, Vec<Expression>, Block)),
    FunctionDecl((String, ParList, Block)),
    LocalFuncDecl((String, ParList, Block)),
}

enum Expression {
    Nil,
    False,
    True,
    Numeral([u8; 8]),
    LiteralString(String),
    DotDotDot,
    /// Used for a variable number of arguments in things like functions
    FunctionDef((ParList, Block)),
    PrefixExp(Box<PrefixExp>),
    TableConstructor(Vec<Field>),
    BinaryOp((Box<Expression>, BinOp, Box<Expression>)),
    UnaryOp((UnOp, Box<Expression>)),
}

enum BinOp {
    Add,
    Sub,
    Mult,
    Div,
    IntegerDiv,
    Pow,
    Mod,
    BitAnd,
    BitXor,
    BitOr,
    ShiftRight,
    ShiftLeft,
    Concat,
    LessThan,
    LessEq,
    GreaterThan,
    GreaterEq,
    Equal,
    NotEqual,
    LogicalAnd,
    LogicalOr,
}

enum UnOp {
    Negate,
    LogicalNot,
    Length,
    BitNot,
}

enum PrefixExp {
    Var(Var),
    // FunctionCall(...),
    Exp(Expression),
}

struct ParList(Vec<String>, bool); // boolean flag is true if there are varargs

enum Field {
    BracketedAssign((Expression, Expression)),
    NameAssign((String, Expression)),
    UnnamedAssign(Expression),
}

enum Var {
    NameVar(String),
    BracketVar((Box<PrefixExp>, Expression)),
    DotVar((Box<PrefixExp>, String)),
}

struct Block {
    statements: Vec<Statement>,
    return_stat: Option<Vec<Expression>>,
}

pub struct AST(Block);


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