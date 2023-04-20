#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assignment((Vec<Var>, Vec<Expression>)),
    FunctionCall(FunctionCall),
    Break,
    DoBlock(Block),
    While((Expression, Block)),
    Repeat((Block, Expression)),
    If((Expression, Block, Vec<(Expression, Block)>, Option<Block>)),
    ForNum((String, Expression, Expression, Option<Expression>, Block)), // for i = 1+2+3, ...
    ForGeneric((Vec<String>, Vec<Expression>, Block)),
    FunctionDecl((String, ParList, Block)),
    LocalFuncDecl((String, ParList, Block)),
    Semicolon,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Nil,
    False,
    True,
    Numeral(Numeral),
    LiteralString(String),
    DotDotDot, // Used for a variable number of arguments in things like functions
    FunctionDef((ParList, Block)),
    PrefixExp(Box<PrefixExp>),
    TableConstructor(Vec<Field>),
    BinaryOp((Box<Expression>, BinOp, Box<Expression>)),
    UnaryOp((UnOp, Box<Expression>)),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOp {
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

#[derive(Debug, PartialEq, Clone)]
pub enum UnOp {
    Negate,
    LogicalNot,
    Length,
    BitNot,
}

// In parsing, we store numeral in i64 and f64, but in interpreting, we store them as [u8; 8]
#[derive(Debug, PartialEq, Clone)]
pub enum Numeral {
    Integer(i64),
    Float(f64),
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrefixExp {
    Var(Var),
    FunctionCall(FunctionCall),
    Exp(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionCall {
    Standard((Box<PrefixExp>, Args)),
    Method((Box<PrefixExp>, String, Args)),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Args {
    ExpList(Vec<Expression>),
    TableConstructor(Vec<Field>),
    LiteralString(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParList(pub Vec<String>, pub bool); // boolean flag is true if there are varargs

#[derive(Debug, PartialEq, Clone)]
pub enum Field {
    BracketedAssign((Expression, Expression)),
    NameAssign((String, Expression)),
    UnnamedAssign(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Var {
    NameVar(String),
    BracketVar((Box<PrefixExp>, Expression)),
    DotVar((Box<PrefixExp>, String)),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub return_stat: Option<Vec<Expression>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AST(pub Block);
