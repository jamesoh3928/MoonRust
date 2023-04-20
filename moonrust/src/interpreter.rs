use crate::ast::*;
use crate::interpreter::environment::Env;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::{cell::RefCell, rc::Rc};

pub mod environment;

#[derive(Debug, PartialEq)]
pub enum LuaVal<'a> {
    LuaTable(Table<'a>),
    LuaNil,
    LuaBool(bool),
    LuaNum([u8; 8]), // Represent numerals as an array of 8 bytes
    LuaString(String),
    Function(LuaFunction<'a>),
}

// Lua function captures environment in function call
#[derive(Debug, PartialEq)]
pub struct LuaFunction<'a> {
    par_list: &'a ParList,
    block: &'a Block,
}

// Wrapper around LuaVal to allow multiple owners
#[derive(Debug, PartialEq, Clone)]
pub struct LuaValue<'a>(Rc<LuaVal<'a>>);
impl<'a> LuaValue<'a> {
    pub fn new(val: LuaVal<'a>) -> Self {
        LuaValue(Rc::new(val))
    }

    pub fn clone(&self) -> LuaValue<'a> {
        LuaValue(Rc::clone(&self.0))
    }

    pub fn is_false(&self) -> bool {
        // All values different from nil and false test true
        match &*self.0 {
            LuaVal::LuaNil => true,
            LuaVal::LuaBool(false) => true,
            _ => false,
        }
    }

    pub fn is_true(&self) -> bool {
        !self.is_false()
    }
}

// TODO: Or use hashmap representation?
#[derive(Debug, PartialEq)]
pub struct Table<'a>(RefCell<Vec<(LuaValue<'a>, LuaValue<'a>)>>);

impl AST {
    pub fn exec<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<(), ASTExecError> {
        self.0.exec(env)?;
        Ok(())
    }
}

impl Block {
    fn exec<'a, 'b>(
        &'a self,
        env: &'b mut Env<'a>,
    ) -> Result<Option<Vec<LuaValue<'a>>>, ASTExecError> {
        let return_vals = self.exec_without_pop(env)?;
        // Remove environment when exiting a scope
        env.pop_local_env();

        Ok(return_vals)
    }

    fn exec_without_pop<'a, 'b>(
        &'a self,
        env: &'b mut Env<'a>,
    ) -> Result<Option<Vec<LuaValue<'a>>>, ASTExecError> {
        // Extend environment when entering a new scope
        env.extend_local_env();

        // Execute each statement
        for statement in &self.statements {
            let return_vals = statement.exec(env)?;
            if return_vals.is_none() {
                // Break statement
                return Ok(None);
            }
        }

        // Optional return statement
        let explist = match &self.return_stat {
            Some(explist) => explist,
            // Returning empty vector means there was no return statement
            None => return Ok(Some(vec![])),
        };

        let mut return_vals = vec![LuaValue::new(LuaVal::LuaNil); explist.len()];
        let mut i = 0;
        for exp in explist.iter() {
            return_vals[i] = exp.eval(env).unwrap();
            i += 1;
        }

        Ok(Some(return_vals))
    }
}

impl Statement {
    fn exec<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<Option<Vec<LuaValue>>, ASTExecError> {
        match self {
            Statement::Semicolon => {
                // Do nothing
            }
            Statement::Assignment((varlist, explist)) => {
                // TODO: add local assignments as well
                // If there are more values than needed, the excess values are thrown away.
                let mut results = Vec::with_capacity(varlist.len());
                for i in 0..varlist.len() {
                    let var = &varlist[i];
                    // If there are fewer values than needed, the list is extended with nil's
                    let val = if i < explist.len() {
                        explist[i].eval(env).unwrap()
                    } else {
                        LuaValue::new(LuaVal::LuaNil)
                    };

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
                    env.insert_global(name.clone(), val);
                }
            }
            Statement::FunctionCall(funcall) => {
                // Returned values are thrown away for statement function call
                funcall.exec(env)?;
            }
            Statement::Break => {
                // Terminates the execution of a while, repeat, or for loop
                return Ok(None);
            }
            Statement::DoBlock(block) => {
                // TODO: add test for this
                match block.exec(env) {
                    Ok(val) => match val {
                        Some(val) => {
                            return Ok(Some(val));
                        }
                        None => {
                            panic!("Break statement can only be used inside a while, repeat, or for loop");
                        }
                    },
                    Err(err) => {
                        return Err(err);
                    }
                }
            }
            Statement::While((exp, block)) => {
                // Execute block until exp returns false
                // Local variables are lost in each iteration
                // TODO: handle break statement
                while exp.eval(env)?.is_true() {
                    let return_vals = block.exec(env)?;
                    match return_vals {
                        Some(return_vals) => {
                            if !return_vals.is_empty() {
                                return Ok(Some(return_vals));
                            }
                        }
                        None => {
                            // Break statement (exiting loop now so return empty vector)
                            return Ok(Some(vec![]));
                        }
                    }
                }
            }
            Statement::Repeat((block, exp)) => {
                // condition can refer to local variables declared inside the loop block
                loop {
                    let return_vals = block.exec_without_pop(env)?;
                    match return_vals {
                        Some(return_vals) => {
                            if !return_vals.is_empty() {
                                return Ok(Some(return_vals));
                            }
                        }
                        None => {
                            // Break statement (exiting loop now so return empty vector)
                            return Ok(Some(vec![]));
                        }
                    }
                    if exp.eval(env)?.is_true() {
                        break;
                    }
                    // Pop local environment before next iteration
                    env.pop_local_env();
                }
            }
            Statement::If((exp, block, elseifs, elseblock)) => {
                let condition = exp.eval(env)?;
                if condition.is_true() {
                    block.exec(env)?;
                } else {
                    // Do elseifs
                    for (exp, block) in elseifs {
                        let condition = exp.eval(env)?;
                        if condition.is_true() {
                            block.exec(env)?;
                            return Ok(Some(vec![]));
                        }
                    }
                    if let Some(elseblock) = elseblock {
                        elseblock.exec(env)?;
                    }
                }
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

        Ok(Some(vec![]))
    }
}

impl Expression {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<LuaValue<'a>, ASTExecError> {
        let val = match self {
            Expression::Nil => LuaValue::new(LuaVal::LuaNil),
            Expression::False => LuaValue::new(LuaVal::LuaBool(false)),
            Expression::True => LuaValue::new(LuaVal::LuaBool(true)),
            Expression::Numeral(n) => match n {
                Numeral::Integer(i) => LuaValue::new(LuaVal::LuaNum(i.to_be_bytes())),
                Numeral::Float(f) => LuaValue::new(LuaVal::LuaNum(f.to_be_bytes())),
            },
            Expression::LiteralString(s) => LuaValue::new(LuaVal::LuaString(s.clone())),
            // TODO: DotDotDot? maybe skip it for now
            Expression::DotDotDot => unimplemented!(),
            Expression::FunctionDef((par_list, block)) => {
                LuaValue::new(LuaVal::Function(LuaFunction { par_list, block }))
            }
            Expression::PrefixExp(prefixexp) => prefixexp.eval(env)?,
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
    fn eval<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<LuaValue<'a>, ASTExecError> {
        match self {
            PrefixExp::Var(var) => {
                match var {
                    Var::NameVar(name) => match env.get(&name) {
                        Some(val) => Ok(val.clone()),
                        None => {
                            return Err(ASTExecError(format!(
                                "Variable {} is not defined in current scope",
                                name
                            )))
                        }
                    },
                    Var::BracketVar((name, exp)) => {
                        // TODO: implement after table
                        unimplemented!()
                    }
                    Var::DotVar((name, field)) => {
                        // TODO: implement after table
                        unimplemented!()
                    }
                }
            }
            PrefixExp::FunctionCall(funcall) => {
                // Call function and check if there is return value
                let return_vals = funcall.exec(env)?;
                if return_vals.len() != 1 {
                    // TODO: double check how to deal when return value is not just 1
                    return Err(ASTExecError(format!(
                        "Function call did not return a value."
                    )));
                } else {
                    Ok(return_vals[0].clone())
                }
            }
            PrefixExp::Exp(exp) => Ok(exp.eval(env)?),
        }
    }
}

impl FunctionCall {
    fn exec<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<Vec<LuaValue<'a>>, ASTExecError> {
        match self {
            FunctionCall::Standard((func, args)) => {
                let func = (*func).eval(env)?;
                let rc = func.0;
                match rc.as_ref() {
                    LuaVal::Function(LuaFunction { par_list, block }) => {
                        // Evaluate arguments first
                        let args = match args {
                            Args::ExpList(exps_list) => {
                                let mut args = Vec::with_capacity(exps_list.len());
                                for exp in exps_list.iter() {
                                    args.push(exp.eval(env)?);
                                }
                                args
                            }
                            Args::TableConstructor(table) => {
                                // TODO: implement after table (single argument of table)
                                unimplemented!()
                            }
                            Args::LiteralString(s) => {
                                vec![LuaValue::new(LuaVal::LuaString(s.clone()))]
                            }
                        };

                        // Extend environment with function arguments
                        env.extend_local_env();
                        let par_length = par_list.0.len();
                        let arg_length = args.len();
                        for i in 0..par_length {
                            // Arguments are locally scoped
                            if i >= arg_length {
                                env.insert_local(
                                    par_list.0[i].clone(),
                                    LuaValue::new(LuaVal::LuaNil),
                                );
                            } else {
                                env.insert_local(par_list.0[i].clone(), args[i].clone());
                            }
                        }

                        let result = block.exec(env)?;

                        // Remove arguments from the environment
                        env.pop_local_env();
                        match result {
                            Some(vals) => Ok(vals),
                            None => panic!(
                                "Break statement can be only used in while, repeat, or for loop"
                            ),
                        }
                    }
                    _ => {
                        return Err(ASTExecError(format!(
                            "Cannot call non-function value with arguments."
                        )))
                    }
                }
            }
            FunctionCall::Method((object, method_name, args)) => {
                // TODO: understand object in Lua?
                unimplemented!()
            }
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

    // Helper functions
    fn var_exp(name: &str) -> Expression {
        Expression::PrefixExp(Box::new(PrefixExp::Var(Var::NameVar(name.to_string()))))
    }
    fn lua_integer<'a>(n: i64) -> LuaValue<'a> {
        LuaValue::new(LuaVal::LuaNum(n.to_be_bytes()))
    }
    fn lua_float<'a>(n: f64) -> LuaValue<'a> {
        LuaValue::new(LuaVal::LuaNum(n.to_be_bytes()))
    }
    fn lua_nil<'a>() -> LuaValue<'a> {
        LuaValue::new(LuaVal::LuaNil)
    }
    fn lua_false<'a>() -> LuaValue<'a> {
        LuaValue::new(LuaVal::LuaBool(false))
    }
    fn lua_true<'a>() -> LuaValue<'a> {
        LuaValue::new(LuaVal::LuaBool(true))
    }
    fn lua_string<'a>(s: &str) -> LuaValue<'a> {
        LuaValue::new(LuaVal::LuaString(s.to_string()))
    }
    fn lua_function<'a>(par_list: &'a ParList, block: &'a Block) -> LuaValue<'a> {
        LuaValue::new(LuaVal::Function(LuaFunction { par_list, block }))
    }

    // TODO split into multiple tests
    #[test]
    fn test_eval_exp_nil() {
        let mut env = Env::new();

        // Nil
        let exp_nil = Expression::Nil;
        assert_eq!(exp_nil.eval(&mut env), Ok(lua_nil()));
    }

    #[test]
    fn test_eval_exp_bool() {
        let mut env = Env::new();

        // Boolean
        let exp_false = Expression::False;
        let exp_true = Expression::True;
        assert_eq!(exp_false.eval(&mut env), Ok(lua_false()));
        assert_eq!(exp_true.eval(&mut env), Ok(lua_true()));
    }

    #[test]
    fn test_eval_exp_int() {
        let mut env = Env::new();

        // Integer
        let num: i64 = 10;
        let exp_int = Expression::Numeral(Numeral::Integer(num));
        assert_eq!(exp_int.eval(&mut env), Ok(lua_integer(num)));
    }

    #[test]
    fn test_eval_exp_float() {
        let mut env = Env::new();

        // Float
        let num: f64 = 10.04;
        let exp_float = Expression::Numeral(Numeral::Float(num));
        assert_eq!(exp_float.eval(&mut env), Ok(lua_float(num)));
    }

    #[test]
    fn test_eval_exp_str() {
        let mut env = Env::new();

        // String
        let exp_str = Expression::LiteralString("Hello World!".to_string());
        assert_eq!(exp_str.eval(&mut env), Ok(lua_string("Hello World!")));
    }

    #[test]
    fn test_eval_exp_func_def() {
        // Test Expression eval method
        let mut env = Env::new();

        // Function definition
        let par_list = ParList(vec![String::from("test")], false);
        let block = Block {
            statements: vec![],
            return_stat: None,
        };
        let exp_func_def = Expression::FunctionDef((par_list.clone(), block.clone()));
        assert_eq!(
            exp_func_def.eval(&mut env),
            Ok(LuaValue::new(LuaVal::Function(LuaFunction {
                par_list: &par_list,
                block: &block
            })))
        );
    }

    #[test]
    fn test_eval_exp_func_call() {
        // TODO: update test to actually execute some statements
        let mut env = Env::new();

        // Set statements
        let varlist = vec![Var::NameVar("a".to_string()), Var::NameVar("b".to_string())];
        let explist = vec![
            Expression::Numeral(Numeral::Integer(30)),
            Expression::Numeral(Numeral::Integer(20)),
        ];
        let stat = Statement::Assignment((varlist, explist));
        let return_stat = Some(vec![var_exp("test")]);

        let par_list = ParList(vec![String::from("test")], false);
        let block = Block {
            statements: vec![stat],
            return_stat: return_stat,
        };

        env.insert_global(String::from("f"), lua_function(&par_list, &block));
        let args = Args::ExpList(vec![Expression::Numeral(Numeral::Integer(100))]);
        let func_call = FunctionCall::Standard((
            Box::new(PrefixExp::Var(Var::NameVar("f".to_string()))),
            args,
        ));
        let exp = PrefixExp::FunctionCall(func_call);

        // f(100) executes a = 30, b = 20, return test
        assert_eq!(
            exp.eval(&mut env),
            Ok(lua_integer(100))
        );
    }

    #[test]
    fn test_exec_stat_assign() {
        // Test Statement exec method
        let mut env = Env::new();

        // Assignment
        let a: i64 = 10;
        let b: i64 = 20;
        let varlist = vec![
            Var::NameVar("a".to_string()),
            Var::NameVar("b".to_string()),
            Var::NameVar("a".to_string()),
        ];
        let explist = vec![
            Expression::Numeral(Numeral::Integer(30)),
            Expression::Numeral(Numeral::Integer(20)),
            Expression::Numeral(Numeral::Integer(10)),
        ];
        let stat = Statement::Assignment((varlist, explist));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(*env.get("a").unwrap().0, LuaVal::LuaNum(a.to_be_bytes()));
        assert_eq!(*env.get("b").unwrap().0, LuaVal::LuaNum(b.to_be_bytes()));

        // varlist.len > explist.len
        let a: i64 = 30;
        let b: i64 = 20;
        let varlist = vec![
            Var::NameVar("a".to_string()),
            Var::NameVar("b".to_string()),
            Var::NameVar("c".to_string()),
        ];
        let explist = vec![
            Expression::Numeral(Numeral::Integer(30)),
            Expression::Numeral(Numeral::Integer(20)),
        ];
        let stat = Statement::Assignment((varlist, explist));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(*env.get("a").unwrap().0, LuaVal::LuaNum(a.to_be_bytes()));
        assert_eq!(*env.get("b").unwrap().0, LuaVal::LuaNum(b.to_be_bytes()));
        assert_eq!(*env.get("c").unwrap().0, LuaVal::LuaNil);

        // varlist.len < explist.len
        let a: i64 = 30;
        let b: i64 = 20;
        let varlist = vec![Var::NameVar("a".to_string()), Var::NameVar("b".to_string())];
        let explist = vec![
            Expression::Numeral(Numeral::Integer(30)),
            Expression::Numeral(Numeral::Integer(20)),
            Expression::Numeral(Numeral::Integer(10)),
        ];
        let stat = Statement::Assignment((varlist, explist));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(*env.get("a").unwrap().0, LuaVal::LuaNum(a.to_be_bytes()));
        assert_eq!(*env.get("b").unwrap().0, LuaVal::LuaNum(b.to_be_bytes()));
    }

    #[test]
    fn test_exec_stat_local_assign() {
        // TODO: local assignment
    }

    #[test]
    fn test_exec_stat_func_call() {
        let mut env = Env::new();
        let a: i64 = 10;
        env.insert_global(
            "a".to_string(),
            lua_integer(a),
        );

        // Simple function with side effect (a = 10.04)
        let varlist = vec![Var::NameVar("a".to_string())];
        let num: f64 = 10.04;
        let exp_float = Expression::Numeral(Numeral::Float(num));
        let explist = vec![exp_float];
        let stat = Statement::Assignment((varlist, explist));
        let return_stat = None;
        let block = Block {
            statements: vec![stat],
            return_stat: return_stat,
        };
        let par_list = ParList(vec![], false);
        env.insert_global(
            "f".to_string(),
            lua_function(&par_list, &block),
        );
        // Function call statement
        let func_prefix = PrefixExp::Var(Var::NameVar("f".to_string()));
        let args = Args::ExpList(vec![]);
        let func_call = FunctionCall::Standard((Box::new(func_prefix), args));
        let func_call_stat = Statement::FunctionCall(func_call);

        assert_eq!(func_call_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            env.get("a"),
            Some(&lua_float(num))
        );
    }

    #[test]
    fn test_exec_stat_if() {
        let mut env = Env::new();
        let condition = Expression::Numeral(Numeral::Integer(0));
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(10))],
            ))],
            return_stat: None,
        };
        let if_stat = Statement::If((condition, block, vec![], None));
        assert_eq!(if_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            env.get("a"),
            Some(&lua_integer(10))
        );
    }

    #[test]
    fn test_exec_stat_else() {
        let mut env = Env::new();
        let condition = Expression::False;
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(10))],
            ))],
            return_stat: None,
        };
        let else_block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(20))],
            ))],
            return_stat: None,
        };
        let if_stat = Statement::If((condition, block, vec![], Some(else_block)));
        assert_eq!(if_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            env.get("a"),
            Some(&lua_integer(20))
        );
    }

    #[test]
    fn test_exec_stat_elseif() {
        let mut env = Env::new();
        let condition = Expression::False;
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(10))],
            ))],
            return_stat: None,
        };
        let else_ifs = vec![(
            Expression::True,
            Block {
                statements: vec![Statement::Assignment((
                    vec![Var::NameVar("a".to_string())],
                    vec![Expression::Numeral(Numeral::Integer(20))],
                ))],
                return_stat: None,
            },
        )];
        let else_block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(30))],
            ))],
            return_stat: None,
        };
        let if_stat = Statement::If((condition, block, else_ifs, Some(else_block)));
        assert_eq!(if_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            env.get("a"),
            Some(&lua_integer(20))
        );
    }

    #[test]
    fn test_exec_stat_doblock() {
        let mut env = Env::new();
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(10))],
            ))],
            return_stat: Some(vec![Expression::PrefixExp(Box::new(PrefixExp::Var(
                Var::NameVar("a".to_string()),
            )))]),
        };
        let do_block = Statement::DoBlock(block);
        assert_eq!(
            do_block.exec(&mut env),
            Ok(Some(vec![LuaValue::new(LuaVal::LuaNum(
                10i64.to_be_bytes()
            ))]))
        );
        assert_eq!(
            env.get("a"),
            Some(&lua_integer(10))
        );
    }

    #[test]
    fn test_exec_stat_while_break() {
        let mut env = Env::new();
        let condition = Expression::True;
        let block = Block {
            statements: vec![
                Statement::Assignment((
                    vec![Var::NameVar("a".to_string())],
                    vec![Expression::Numeral(Numeral::Integer(10))],
                )),
                Statement::Break,
            ],
            return_stat: None,
        };
        let while_stat = Statement::While((condition, block));
        assert_eq!(while_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            env.get("a"),
            Some(&lua_integer(10))
        );
    }

    #[test]
    fn test_exec_stat_while_return() {
        let mut env = Env::new();
        let condition = Expression::True;
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(10))],
            ))],
            return_stat: Some(vec![var_exp("a")]),
        };
        let while_stat = Statement::While((condition, block));
        assert_eq!(while_stat.exec(&mut env), Ok(Some(vec![lua_integer(10)])));
        assert_eq!(env.get("a"), Some(&lua_integer(10)));
    }

    #[test]
    fn test_exec_stat_while() {
        // TODO: increment few times when binary expression is implemented
        // let mut env = Env::new();
        // let condition = Expression::False;
        // let block = Block {
        //     statements: vec![Statement::Assignment((
        //         vec![Var::NameVar("a".to_string())],
        //         vec![Expression::Numeral(Numeral::Integer(10))],
        //     ))],
        //     return_stat: None,
        // };
    }
}
