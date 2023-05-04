use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assignment((Vec<Var>, Vec<Expression>, bool)), // bool flag: true if local assn, false otherwise
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

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Assignment((vars, exps, local)) => {
                if *local {
                    write!(f, "local ")?;
                }
                format_list(vars, f, false)?;
                write!(f, " = ")?;
                format_list(exps, f, false)?;
            }
            Statement::FunctionCall(fncall) => fncall.fmt(f)?,
            Statement::Break => write!(f, "break")?,
            Statement::DoBlock(block) => {
                writeln!(f, "do")?;
                block.fmt(f)?;
                write!(f, "end")?;
            }
            Statement::While((exp, block)) => {
                write!(f, "while ")?;
                exp.fmt(f)?;
                writeln!(f, " do")?;
                block.fmt(f)?;
                write!(f, "end")?;
            }
            Statement::Repeat((block, exp)) => {
                writeln!(f, "repeat")?;
                block.fmt(f)?;
                write!(f, "until ")?;
                exp.fmt(f)?;
            }
            Statement::If((cond, then_block, elseifs, else_block)) => {
                write!(f, "if ")?;
                cond.fmt(f)?;
                writeln!(f, " then")?;
                then_block.fmt(f)?;
                elseifs.iter().try_for_each(|elseif| {
                    write!(f, "elseif ")?;
                    elseif.0.fmt(f)?;
                    writeln!(f, " then")?;
                    elseif.1.fmt(f)
                })?;
                if let Some(elseb) = else_block {
                    writeln!(f, "else")?;
                    elseb.fmt(f)?;
                }
                write!(f, " end")?;
            }
            Statement::ForNum((name, exp1, exp2, maybe_exp3, block)) => {
                write!(f, "for ")?;
                name.fmt(f)?;
                write!(f, " = ")?;
                exp1.fmt(f)?;
                write!(f, ", ")?;
                exp2.fmt(f)?;
                if let Some(exp3) = maybe_exp3 {
                    write!(f, ", ")?;
                    exp3.fmt(f)?;
                }
                writeln!(f, " do")?;
                block.fmt(f)?;
                write!(f, "end")?;
            }
            Statement::ForGeneric((names, exps, block)) => {
                write!(f, "for ")?;
                format_list(names, f, false)?;
                write!(f, " in ")?;
                format_list(exps, f, false)?;
                writeln!(f, " do")?;
                block.fmt(f)?;
                write!(f, "end")?;
            }
            Statement::FunctionDecl((name, parlist, block)) => {
                write!(f, "function ")?;
                name.fmt(f)?;
                write!(f, "(")?;
                parlist.fmt(f)?;
                writeln!(f, ")")?;
                block.fmt(f)?;
                write!(f, "end")?;
            }
            Statement::LocalFuncDecl((name, parlist, block)) => {
                write!(f, "local function ")?;
                name.fmt(f)?;
                write!(f, "(")?;
                parlist.fmt(f)?;
                writeln!(f, ")")?;
                block.fmt(f)?;
                write!(f, "end")?;
            }
            Statement::Semicolon => write!(f, ";")?,
        }
        writeln!(f)
    }
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

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Nil => write!(f, "nil"),
            Expression::False => write!(f, "false"),
            Expression::True => write!(f, "true"),
            Expression::Numeral(numeral) => numeral.fmt(f),
            Expression::LiteralString(string) => {
                write!(f, "\"")?;
                string.fmt(f)?;
                write!(f, "\"")
            }
            Expression::DotDotDot => write!(f, "..."),
            Expression::FunctionDef((parlist, block)) => {
                write!(f, "function")?;
                write!(f, "(")?;
                parlist.fmt(f)?;
                writeln!(f, ")")?;
                block.fmt(f)?;
                writeln!(f, "end")
            }
            Expression::PrefixExp(pexp) => pexp.fmt(f),
            Expression::TableConstructor(fields) => {
                writeln!(f, "{{")?;
                format_list(fields, f, true)?;
                writeln!(f, "}}")
            }
            Expression::BinaryOp((exp1, binop, exp2)) => {
                exp1.fmt(f)?;
                binop.fmt(f)?;
                exp2.fmt(f)
            }
            Expression::UnaryOp((unop, exp)) => {
                unop.fmt(f)?;
                exp.fmt(f)
            }
        }
    }
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

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, " + "),
            BinOp::Sub => write!(f, " - "),
            BinOp::Mult => write!(f, " * "),
            BinOp::Div => write!(f, " / "),
            BinOp::IntegerDiv => write!(f, " // "),
            BinOp::Pow => write!(f, " ^ "),
            BinOp::Mod => write!(f, " % "),
            BinOp::BitAnd => write!(f, " & "),
            BinOp::BitXor => write!(f, " ~ "),
            BinOp::BitOr => write!(f, " | "),
            BinOp::ShiftRight => write!(f, " >> "),
            BinOp::ShiftLeft => write!(f, " << "),
            BinOp::Concat => write!(f, " .. "),
            BinOp::LessThan => write!(f, " < "),
            BinOp::LessEq => write!(f, " <= "),
            BinOp::GreaterThan => write!(f, " > "),
            BinOp::GreaterEq => write!(f, " >= "),
            BinOp::Equal => write!(f, " == "),
            BinOp::NotEqual => write!(f, " ~= "),
            BinOp::LogicalAnd => write!(f, " and "),
            BinOp::LogicalOr => write!(f, " or "),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnOp {
    Negate,
    LogicalNot,
    Length,
    BitNot,
}

impl Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Negate => write!(f, "-"),
            Self::LogicalNot => write!(f, "not"),
            Self::Length => write!(f, "#"),
            Self::BitNot => write!(f, "~"),
        }
    }
}

// In parsing, we store numeral in i64 and f64, but in interpreting, we store them as [u8; 8]
#[derive(Debug, PartialEq, Clone)]
pub enum Numeral {
    Integer(i64),
    Float(f64),
}

impl Display for Numeral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(i) => i.fmt(f),
            Self::Float(fl) => fl.fmt(f),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrefixExp {
    Var(Var),
    FunctionCall(FunctionCall),
    Exp(Expression),
}

impl Display for PrefixExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Var(var) => var.fmt(f),
            Self::FunctionCall(fncall) => fncall.fmt(f),
            Self::Exp(exp) => exp.fmt(f),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FunctionCall {
    Standard((Box<PrefixExp>, Args)),
    Method((Box<PrefixExp>, String, Args)),
}

impl Display for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard((pexp, args)) => {
                pexp.fmt(f)?;
                args.fmt(f)
            }
            Self::Method((pexp, name, args)) => {
                pexp.fmt(f)?;
                write!(f, ":")?;
                name.fmt(f)?;
                args.fmt(f)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Args {
    ExpList(Vec<Expression>),
    TableConstructor(Vec<Field>),
    LiteralString(String),
}

impl Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpList(exps) => {
                write!(f, "(")?;
                format_list(exps, f, false)?;
                write!(f, ")")
            }
            Self::TableConstructor(fields) => {
                writeln!(f, "{{")?;
                format_list(fields, f, true)?;
                writeln!(f, "}}")
            }
            Self::LiteralString(string) => {
                write!(f, "\"")?;
                string.fmt(f)?;
                write!(f, "\"")
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParList(pub Vec<String>, pub bool); // boolean flag is true if there are varargs

impl Display for ParList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            if self.1 {
                return write!(f, "...");
            }
            Ok(())
        } else {
            format_list(&self.0, f, false)?;

            if self.1 {
                write!(f, ", ...")
            } else {
                Ok(())
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Field {
    BracketedAssign((Expression, Expression)),
    NameAssign((String, Expression)),
    UnnamedAssign(Expression),
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BracketedAssign((exp1, exp2)) => {
                write!(f, "[")?;
                exp1.fmt(f)?;
                write!(f, "] = ")?;
                exp2.fmt(f)
            }
            Self::NameAssign((name, exp)) => {
                name.fmt(f)?;
                write!(f, " = ")?;
                exp.fmt(f)
            }
            Self::UnnamedAssign(exp) => exp.fmt(f),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Var {
    NameVar(String),
    BracketVar((Box<PrefixExp>, Expression)),
    DotVar((Box<PrefixExp>, String)),
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NameVar(name) => name.fmt(f),
            Self::BracketVar((pexp, exp)) => {
                pexp.fmt(f)?;
                write!(f, "[")?;
                exp.fmt(f)?;
                write!(f, "]")
            }
            Self::DotVar((pexp, name)) => {
                pexp.fmt(f)?;
                write!(f, ".")?;
                name.fmt(f)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub return_stat: Option<Vec<Expression>>,
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.statements.iter().try_for_each(|statement| {
            write!(f, "    ")?;
            statement.fmt(f)
        })?;

        if let Some(ret) = &self.return_stat {
            write!(f, "    return ")?;
            ret.iter().try_for_each(|exp| exp.fmt(f))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AST(pub Block);

impl Display for AST {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
            .statements
            .iter()
            .try_for_each(|statement| statement.fmt(f))?;

        if let Some(ret) = &self.0.return_stat {
            ret.iter().try_for_each(|exp| exp.fmt(f))?;
        }

        writeln!(f)
    }
}

fn format_list(
    elements: &[impl Display],
    f: &mut std::fmt::Formatter<'_>,
    lines: bool,
) -> std::fmt::Result {
    let mut el_iter = elements.iter().peekable();
    while let Some(element) = el_iter.next() {
        element.fmt(f)?;
        if el_iter.peek().is_some() {
            if lines {
                writeln!(f, ", ")?;
            } else {
                write!(f, ", ")?;
            }
        }
    }
    Ok(())
}
