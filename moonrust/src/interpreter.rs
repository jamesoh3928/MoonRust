use crate::ast::*;
use crate::interpreter::environment::Env;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub mod environment;

enum LuaValue {
    LuaTable(Table),
    LuaNil,
    LuaBool(bool),
    LuaNum([u8; 8]), // Represent numerals as an array of 8 bytes
    LuaString(String),
    Function(LuaFunction),
}

pub struct LuaFunction {
    name: String,
    arity: usize, // The number of arguments
    statement: Vec<AST>,
}

pub struct LuaVar(Rc<RefCell<LuaValue>>);

struct Table(HashMap<LuaValue, Rc<RefCell<LuaValue>>>);

impl AST {
    pub fn exec(self, env: &mut Env) -> Result<(), ASTExecError> {
        self.0.exec(env)?;
        Ok(())
    }
}

impl Block {
    fn exec(self, env: &mut Env) -> Result<Vec<LuaValue>, ASTExecError> {
        // Extend environment when entering a new scope
        env.extend_env();

        for statement in &self.statements {
            statement.exec(env)?;
        }
        match self.return_stat {
            Some(explist) => {
                // Return vector of lua values
                Ok(explist
                    .into_iter()
                    .map(|exp| exp.eval(env))
                    .collect::<Vec<LuaValue>>())
            }
            None => {
                Ok(vec![])
            }
        }
    }
}

impl Statement {
    fn exec(&self, env: &mut Env) -> Result<(), ASTExecError> {
        // TODO: implement exec for each statement (probably huge match statement)
        match self {
            Statement::Assignment((varlist, explist)) => {
                unimplemented!()
            }
            Statement::FunctionCall(funcall) => {
                unimplemented!()
            }
            Statement::Break => {
                unimplemented!()
            }
            Statement::DoBlock(block) => {
                unimplemented!()
            }
            Statement::While((exp, block)) => {
                unimplemented!()
            }
            Statement::Repeat((block, exp)) => {
                unimplemented!()
            }
            Statement::If((exp, block, elseifs, elseblock)) => {
                unimplemented!()
            }
            Statement::ForNum((name, exp1, exp2, exp3, block)) => {
                unimplemented!()
            }
            Statement::ForGeneric((names, explist, block)) => {
                unimplemented!()
            }
            Statement::FunctionDecl((name, parlist, block)) => {
                unimplemented!()
            }
            Statement::LocalFuncDecl((name, parlist, block)) => {
                unimplemented!()
            }
        };

        Ok(())
    }
}

impl Expression {
    fn eval(self, env: &mut Env) -> LuaValue {
        // TODO: implement eval for each expression (probably huge match statement)
        match self {
            Expression::Nil => LuaValue::LuaNil,
            Expression::False => LuaValue::LuaBool(false),
            Expression::True => LuaValue::LuaBool(true),
            // TODO: convert integer/float to byte array
            Expression::Numeral(n) => unimplemented!(),
            Expression::LiteralString(s) => LuaValue::LuaString(s),
            Expression::DotDotDot => unimplemented!(),
            Expression::FunctionDef((parlist, block)) => unimplemented!(),
            Expression::PrefixExp(prefixexp) => unimplemented!(),
            Expression::TableConstructor(fields) => unimplemented!(),
            Expression::BinaryOp((left, op, right)) => unimplemented!(),
            Expression::UnaryOp((op, exp)) => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ASTExecError(String);
impl Display for ASTExecError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
