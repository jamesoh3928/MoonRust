pub enum Statement {
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

pub enum Expression {
    Nil,
    False,
    True,
    Numeral([u8; 8]),
    LiteralString(String),
    DotDotDot, // Used for a variable number of arguments in things like functions
    FunctionDef((ParList, Block)),
    PrefixExp(Box<PrefixExp>),
    TableConstructor(Vec<Field>),
    BinaryOp((Box<Expression>, BinOp, Box<Expression>)),
    UnaryOp((UnOp, Box<Expression>)),
}

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

pub enum UnOp {
    Negate,
    LogicalNot,
    Length,
    BitNot,
}

pub enum PrefixExp {
    Var(Var),
    // FunctionCall(Expression::DotDotDot), // TODO: @Matt question? Were you expecting DotDotDot to be here?
    Exp(Expression),
}

pub struct ParList(Vec<String>, bool); // boolean flag is true if there are varargs

pub enum Field {
    BracketedAssign((Expression, Expression)),
    NameAssign((String, Expression)),
    UnnamedAssign(Expression),
}

pub enum Var {
    NameVar(String),
    BracketVar((Box<PrefixExp>, Expression)),
    DotVar((Box<PrefixExp>, String)),
}

pub struct Block {
    pub statements: Vec<Statement>,
    pub return_stat: Option<Vec<Expression>>,
}

pub struct ASTParseError(String);

pub struct AST(pub Block);
