use crate::ast::*;
use crate::interpreter::environment::Env;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::{cell::RefCell, rc::Rc};

pub mod environment;

#[derive(Debug, PartialEq)]
pub enum LuaVal {
    LuaTable(Table),
    LuaNil,
    LuaBool(bool),
    LuaNum([u8; 8]), // Represent numerals as an array of 8 bytes
    LuaString(String),
    Function(LuaFunction),
}

// Lua function captures environment in function call
#[derive(Debug, PartialEq)]
pub struct LuaFunction {
    par_list: ParList,
    block: Block,
}

// Wrapper around LuaVal to allow multiple owners
#[derive(Debug, PartialEq, Clone)]
pub struct LuaValue(Rc<RefCell<LuaVal>>);
impl LuaValue {
    pub fn new(val: LuaVal) -> Self {
        LuaValue(Rc::new(RefCell::new(val)))
    }

    pub fn clone(&self) -> LuaValue {
        LuaValue(Rc::clone(&self.0))
    }
}

// TODO: Or use hashmap representation?
#[derive(Debug, PartialEq)]
pub struct Table(Vec<(LuaVal, LuaValue)>);

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

        // Execute each statement
        for statement in self.statements {
            statement.exec(env)?;
        }

        // Optional return statement
        let mut explist = match self.return_stat {
            Some(explist) => explist,
            None => vec![],
        };

        let mut return_val = vec![LuaValue::new(LuaVal::LuaNil); explist.len()];
        let length = explist.len();
        explist.reverse();
        for i in 0..length {
            let exp = explist.pop().unwrap();
            return_val[i] = exp.eval(env)?;
        }

        // Remove environment when exiting a scope
        env.pop_env();

        Ok(return_val)
    }
}

impl Statement {
    fn exec(self, env: &mut Env) -> Result<(), ASTExecError> {
        match self {
            Statement::Assignment((mut varlist, mut explist)) => {
                // If there are more values than needed, the excess values are thrown away.
                while varlist.len() < explist.len() {
                    varlist.pop();
                }
                // If there are fewer values than needed, the list is extended with nil's
                while varlist.len() > explist.len() {
                    explist.push(Expression::Nil);
                }
                // First get all the results instead of inserting into the environment
                // to ensure that all expressions are evaluated with original values
                let mut results = vec![];
                while explist.len() > 0 {
                    let val = explist.pop().unwrap().eval(env)?;
                    let var = varlist.pop().unwrap();
                    match var {
                        Var::NameVar(name) => {
                            results.push((name, val));
                        }
                        // TODO: assignments for tables
                        Var::BracketVar((name, exp)) => {
                            unimplemented!()
                        }
                        Var::DotVar((name, field)) => {
                            unimplemented!()
                        }
                    }
                }

                // Insert into the environment
                for (name, val) in results {
                    env.insert(name, val);
                }
            }
            Statement::FunctionCall(funcall) => {
                unimplemented!()
            }
            Statement::Break => {
                // TODO: should just break innermost loop for while, repeat, or for (how can we do this?)
                unimplemented!()
            }
            Statement::DoBlock(block) => {
                unimplemented!()
            }
            Statement::While((exp, block)) => {
                // TODO: have to do it so that we don't lose exp and block....
                // Execute block until exp returns false
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
    fn eval(self, env: &mut Env) -> Result<LuaValue, ASTExecError> {
        let val = match self {
            Expression::Nil => LuaValue::new(LuaVal::LuaNil),
            Expression::False => LuaValue::new(LuaVal::LuaBool(false)),
            Expression::True => LuaValue::new(LuaVal::LuaBool(true)),
            Expression::Numeral(n) => match n {
                Numeral::Integer(i) => LuaValue::new(LuaVal::LuaNum(i.to_be_bytes())),
                Numeral::Float(f) => LuaValue::new(LuaVal::LuaNum(f.to_be_bytes())),
            },
            Expression::LiteralString(s) => LuaValue::new(LuaVal::LuaString(s)),
            // TODO: DotDotDot?
            Expression::DotDotDot => unimplemented!(),
            Expression::FunctionDef((par_list, block)) => {
                LuaValue::new(LuaVal::Function(LuaFunction { par_list, block }))
            }
            Expression::PrefixExp(prefixexp) => {
                match *prefixexp {
                    PrefixExp::Var(var) => {
                        match var {
                            Var::NameVar(name) => {
                                match env.get(&name) {
                                    Some(val) => val.clone(),
                                    None => return Err(ASTExecError(format!("Variable {} is not defined in current scope", name))),
                                }
                            }
                            Var::BracketVar((name, exp)) => {
                                // TODO: implement after table
                                unimplemented!()
                            }
                            Var::DotVar((name, field)) => {
                                // TODO: implement after table
                                unimplemented!()
                            }
                        }
                    },
                    PrefixExp::FunctionCall(funcall) => {
                        unimplemented!()
                        // funcall.exec(env)?
                    },
                    PrefixExp::Exp(exp) => {
                        exp.eval(env)?
                    }
                }
            }
            Expression::TableConstructor(fields) => unimplemented!(),
            Expression::BinaryOp((left, op, right)) => {
                unimplemented!()
            }
            Expression::UnaryOp((op, exp)) => unimplemented!(),
        };
        Ok(val)
    }
}

impl PrefixExp {
    fn eval(self, env: &mut Env) -> Result<LuaValue, ASTExecError> {
        match self {
            PrefixExp::Var(var) => {
                match var {
                    Var::NameVar(name) => {
                        match env.get(&name) {
                            Some(val) => Ok(val.clone()),
                            None => return Err(ASTExecError(format!("Variable {} is not defined in current scope", name))),
                        }
                    }
                    Var::BracketVar((name, exp)) => {
                        // TODO: implement after table
                        unimplemented!()
                    }
                    Var::DotVar((name, field)) => {
                        // TODO: implement after table
                        unimplemented!()
                    }
                }
            },
            PrefixExp::FunctionCall(funcall) => {
                // Call function and check if there is return value
                let return_vals = funcall.exec(env)?;
                if return_vals.len() != 1 {
                    // TODO: double check how to deal when return value is not just 1
                    return Err(ASTExecError(format!("Function call did not return a value.")));
                } else {
                    Ok(return_vals[0].clone())
                }
            },
            PrefixExp::Exp(exp) => {
                Ok(exp.eval(env)?)
            }
        }
    }
}

impl FunctionCall {
    fn exec(self, env: &mut Env) -> Result<Vec<LuaValue>, ASTExecError> {
        match self {
            FunctionCall::Standard((func, args)) => {
                let func = (*func).eval(env)?;
                match func {
                    // Check if prefixexp evaluates to a function
                    LuaValue(rc) => {
                        match &mut *rc.borrow_mut() {
                            LuaVal::Function(LuaFunction{par_list, block}) => {
                                // Evaluate arguments first
                                let args = match args {
                                    Args::ExpList(mut exps_list) => {
                                        let mut args = Vec::new();
                                        while exps_list.len() > 0 {
                                            args.push(match exps_list.pop() {
                                                Some(exp) => exp.eval(env)?,
                                                None => return Err(ASTExecError(format!("Cannot call function with empty argument."))),
                                            })
                                        }
                                        args.reverse();
                                        args
                                    },
                                    Args::TableConstructor(table) => {
                                        // TODO: implement after table (single argument of table)
                                        unimplemented!()
                                    },
                                    Args::LiteralString(s) => {
                                        vec![LuaValue::new(LuaVal::LuaString(s))]
                                    },
                                };

                                // Extend environment with function arguments
                                env.extend_env();
                                let par_length = par_list.0.len();
                                let arg_length = args.len();
                                for i in 0..par_length {
                                    if i >= arg_length {
                                        env.insert(par_list.0[i].clone(), LuaValue::new(LuaVal::LuaNil));
                                    } else {
                                        env.insert(par_list.0[i].clone(), args[i].clone());
                                    }
                                }

                                // let result = block.clone().exec(env)?;

                                // Remove arguments from the environment
                                env.pop_env();
                                // Ok(result);

                                unimplemented!()
                            },
                            _ => return Err(ASTExecError(format!("Cannot call non-function value with arguments."))),
                        }
                    },
                    _ => return Err(ASTExecError(format!("Cannot call non-function value with arguments."))),
                }
                
            }, 
            FunctionCall::Method((object, method_name, args)) => {
                // TODO: understand object in Lua?
                unimplemented!()
            },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_exp() {
        // Test Expression eval method
        let mut env = Env::new();

        // Nil
        let exp_nil = Expression::Nil;
        assert_eq!(exp_nil.eval(&mut env), Ok(LuaValue::new(LuaVal::LuaNil)));

        // Boolean
        let exp_false = Expression::False;
        let exp_true = Expression::True;
        assert_eq!(
            exp_false.eval(&mut env),
            Ok(LuaValue::new(LuaVal::LuaBool(false)))
        );
        assert_eq!(
            exp_true.eval(&mut env),
            Ok(LuaValue::new(LuaVal::LuaBool(true)))
        );

        // Integer
        let num: i64 = 10;
        let exp_int = Expression::Numeral(Numeral::Integer(num));
        assert_eq!(
            exp_int.eval(&mut env),
            Ok(LuaValue::new(LuaVal::LuaNum(num.to_be_bytes())))
        );

        // Float
        let num: f64 = 10.04;
        let exp_float = Expression::Numeral(Numeral::Float(num));
        assert_eq!(
            exp_float.eval(&mut env),
            Ok(LuaValue::new(LuaVal::LuaNum(num.to_be_bytes())))
        );

        // String
        let exp_str = Expression::LiteralString("Hello World!".to_string());
        assert_eq!(
            exp_str.eval(&mut env),
            Ok(LuaValue::new(LuaVal::LuaString("Hello World!".to_string())))
        );

        // Function definition
        let par_list = ParList(vec![String::from("test")], false);
        let block = Block {
            statements: vec![],
            return_stat: None,
        };
        let exp_func_def = Expression::FunctionDef((par_list, block));
        assert_eq!(exp_func_def.eval(&mut env), Ok(LuaValue::new(LuaVal::Function(LuaFunction {
            par_list: ParList(vec![String::from("test")], false),
            block: Block {
                statements: vec![],
                return_stat: None,
            }
        }))));
    }

    #[test]
    fn test_exec_stat() {
        // Test Statement exec method
        let mut env = Env::new();

        // Assignment
        let a: i64 = 10;
        let b: i64 = 20;
        let varlist = vec![Var::NameVar("a".to_string()), Var::NameVar("b".to_string())];
        let explist = vec![
            Expression::Numeral(Numeral::Integer(10)),
            Expression::Numeral(Numeral::Integer(20)),
        ];
        let stat = Statement::Assignment((varlist, explist));
        assert_eq!(stat.exec(&mut env), Ok(()));
        assert_eq!(
            *env.get("a").unwrap().0.borrow(),
            LuaVal::LuaNum(a.to_be_bytes())
        );
        assert_eq!(
            *env.get("b").unwrap().0.borrow(),
            LuaVal::LuaNum(b.to_be_bytes())
        );
    }
}
