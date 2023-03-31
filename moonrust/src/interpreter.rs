use crate::ast::*;
use std::fmt;
use std::fmt::{Display, Formatter};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

struct LuaVar(Rc<RefCell<LuaValue>>);

struct Table(HashMap<LuaValue, Rc<RefCell<LuaValue>>>);

// TODO: double check environment implementation
// Dr. Fluet's advice: env: Vec<Table<String, Data>>, type Env = (Table<String, Data>, Vec<Table<String, Data>>)
struct EnvTable(Vec<(String, LuaVar)>);
// Insert None between each EnvTable to represent a new scope
struct Env(Option<Vec<EnvTable>>);

impl AST {
    pub fn exec(&self) -> Result<(), ASTExecError> {
        unimplemented!();
        // self.0.exec();
    }
}

impl Block {
    fn exec(&self) {
        // TODO: keep track of variables in a scope (probably a hashmap)
        for statement in &self.statements {
            statement.exec();
        }
    }
}

impl Statement {
    fn exec(&self) {
        // TODO: implement exec for each statement (probably huge match statement)
        unimplemented!()
    }
}

impl Expression {
    fn eval(&self) -> LuaValue {
        // TODO: implement eval for each expression (probably huge match statement)
        unimplemented!()
    }
}

#[derive(Debug, PartialEq)]
pub struct ASTExecError(String);
impl Display for ASTExecError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}