use crate::ast::*;
use crate::interpreter::environment::Env;
use crate::interpreter::ASTExecError;
use crate::interpreter::LuaFunction;
use crate::interpreter::LuaTable;
use crate::interpreter::LuaVal;
use crate::interpreter::LuaValue;

impl Expression {
    pub fn eval<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<LuaValue<'a>, ASTExecError> {
        let val = match self {
            Expression::Nil => LuaValue::new(LuaVal::LuaNil),
            Expression::False => LuaValue::new(LuaVal::LuaBool(false)),
            Expression::True => LuaValue::new(LuaVal::LuaBool(true)),
            Expression::Numeral(n) => match n {
                Numeral::Integer(i) => LuaValue::new(LuaVal::LuaNum(i.to_be_bytes(), false)),
                Numeral::Float(f) => LuaValue::new(LuaVal::LuaNum(f.to_be_bytes(), true)),
            },
            Expression::LiteralString(s) => LuaValue::new(LuaVal::LuaString(s.clone())),
            // TODO: DotDotDot? maybe skip it for now
            Expression::DotDotDot => unimplemented!(),
            Expression::FunctionDef((par_list, block)) => {
                LuaValue::new(LuaVal::Function(LuaFunction { par_list, block }))
            }
            Expression::PrefixExp(prefixexp) => prefixexp.eval(env)?,
            Expression::TableConstructor(fields) => {
                // TODO
                let mut table = LuaTable::new(fields.len());
                for field in fields.into_iter() {
                    match field {
                        Field::BracketedAssign((exp1, exp2)) => {
                            unimplemented!()
                        }
                        Field::NameAssign((name, exp)) => {
                            unimplemented!()
                        }
                        Field::UnnamedAssign(exp) => {
                            unimplemented!()
                        }
                    }
                }
                unimplemented!()
            }
            Expression::BinaryOp((left, op, right)) => {
                // If both are integers, the operation is performed over integers and the result is an integer.
                // If both are numbers, then they are converted to floats
                match op {
                    BinOp::Add => {
                        let left = left.eval(env)?;
                        let right = right.eval(env)?;
                        match (left.0.as_ref(), right.0.as_ref()) {
                            (
                                LuaVal::LuaNum(bytes1, is_float1),
                                LuaVal::LuaNum(bytes2, is_float2),
                            ) => {
                                if !*is_float1 && !*is_float2 {
                                    // Both are integers
                                    let i1 = i64::from_be_bytes(*bytes1);
                                    let i2 = i64::from_be_bytes(*bytes2);
                                    LuaValue::new(LuaVal::LuaNum((i1 + i2).to_be_bytes(), false))
                                } else if *is_float1 {
                                    // Left is float, right is integer
                                    let f1 = f64::from_be_bytes(*bytes1);
                                    let i2 = i64::from_be_bytes(*bytes2);
                                    LuaValue::new(LuaVal::LuaNum(
                                        (f1 + i2 as f64).to_be_bytes(),
                                        true,
                                    ))
                                } else if *is_float2 {
                                    // Right is float, left is integer
                                    let i1 = i64::from_be_bytes(*bytes1);
                                    let f2 = f64::from_be_bytes(*bytes2);
                                    LuaValue::new(LuaVal::LuaNum(
                                        (i1 as f64 + f2).to_be_bytes(),
                                        true,
                                    ))
                                } else {
                                    // Both are float
                                    let f1 = f64::from_be_bytes(*bytes1);
                                    let f2 = f64::from_be_bytes(*bytes2);
                                    LuaValue::new(LuaVal::LuaNum((f1 + f2).to_be_bytes(), true))
                                }
                            }
                            // TODO: string coercion to numbers (maybe skip for now)
                            _ => {
                                return Err(ASTExecError(format!(
                                    "Cannot add values that are not numbers"
                                )));
                            }
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            Expression::UnaryOp((op, exp)) => unimplemented!(),
        };
        Ok(val)
    }
}

impl PrefixExp {
    pub fn eval<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<LuaValue<'a>, ASTExecError> {
        match self {
            PrefixExp::Var(var) => {
                match var {
                    Var::NameVar(name) => match env.get(&name) {
                        Some(val) => Ok(val.clone()),
                        None => Ok(LuaValue::new(LuaVal::LuaNil)),
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
    pub fn exec<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<Vec<LuaValue<'a>>, ASTExecError> {
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
                            None => Err(ASTExecError(format!(
                                "Break statement can be only used in while, repeat, or for loop"
                            ))),
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
                // TODO: Lua object is basically a table (implement this after table is implemented)
                unimplemented!()
            }
        }
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
        LuaValue::new(LuaVal::LuaNum(n.to_be_bytes(), false))
    }
    fn lua_float<'a>(n: f64) -> LuaValue<'a> {
        LuaValue::new(LuaVal::LuaNum(n.to_be_bytes(), true))
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
        let mut env = Env::new();

        // Set statements
        let varlist = vec![Var::NameVar("a".to_string()), Var::NameVar("b".to_string())];
        let explist = vec![
            Expression::Numeral(Numeral::Integer(30)),
            Expression::Numeral(Numeral::Integer(20)),
        ];
        let stat = Statement::Assignment((varlist, explist, false));
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
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(100)));
    }

    #[test]
    fn test_eval_bin_exp_add() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Add, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(30)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Add, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(30.1)));
    }
}
