use crate::ast::*;
use crate::interpreter::environment::Env;
use crate::interpreter::ASTExecError;
use crate::interpreter::LuaFunction;
use crate::interpreter::LuaVal;
use crate::interpreter::LuaValue;

impl Statement {
    pub fn exec<'a, 'b>(
        &'a self,
        env: &'b mut Env<'a>,
    ) -> Result<Option<Vec<LuaValue>>, ASTExecError> {
        match self {
            Statement::Semicolon => {
                // Do nothing
            }
            Statement::Assignment((varlist, explist, is_local)) => {
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
                        // TODO: implement after table
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
                    if *is_local {
                        // With local keyword, always insert new local variable or overwrite existing
                        env.insert_local(name.clone(), val);
                    } else {
                        if env.get_local(name).is_some() {
                            // Update local variable
                            env.insert_local(name.clone(), val);
                        } else {
                            // Update or insert global variable
                            env.insert_global(name.clone(), val);
                        }
                    }
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
            Statement::DoBlock(block) => match block.exec(env) {
                Ok(val) => match val {
                    Some(val) => {
                        return Ok(Some(val));
                    }
                    None => {
                        return Err(ASTExecError(format!(
                            "Break statement can only be used inside a while, repeat, or for loop"
                        )))
                    }
                },
                Err(err) => {
                    return Err(err);
                }
            },
            Statement::While((exp, block)) => {
                // Execute block until exp returns false
                // Local variables are lost in each iteration
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
                let initial = match exp1.eval(env)? {
                    LuaValue(rc) => match rc.as_ref() {
                        LuaVal::LuaNum(bytes, is_float) => {
                            if *is_float {
                                return Err(ASTExecError(format!(
                                    "Initial value in for loop must be an integer"
                                )));
                            }
                            i64::from_be_bytes(*bytes)
                        }
                        _ => {
                            return Err(ASTExecError(format!(
                                "Initial value in for loop must be an integer"
                            )));
                        }
                    },
                };
                let limit = exp2.eval(env)?;
                let step = match exp3 {
                    Some(exp) => match exp.eval(env)? {
                        LuaValue(rc) => match rc.as_ref() {
                            LuaVal::LuaNum(bytes, is_float) => {
                                if *is_float {
                                    return Err(ASTExecError(format!(
                                        "Step value in for loop must be an integer"
                                    )));
                                }
                                i64::from_be_bytes(*bytes)
                            }
                            _ => {
                                return Err(ASTExecError(format!(
                                    "Step value in for loop must be an integer"
                                )));
                            }
                        },
                        _ => {
                            return Err(ASTExecError(format!(
                                "Step value in for loop must be an integer"
                            )));
                        }
                    },
                    None => 1,
                };

                // Step 0 is not allowed
                if step == 0 {
                    return Err(ASTExecError(format!("Step value in for loop cannot be 0")));
                }

                // continues while the value is less than or equal to the limit
                // (greater than or equal to for a negative step)
                let mut i = initial;
                while if step > 0 {
                    limit.is_greater_or_equal(i)?
                } else {
                    limit.is_less_or_equal(i)?
                } {
                    // Create a new local environment
                    env.extend_local_env();
                    env.insert_local(
                        name.clone(),
                        LuaValue::new(LuaVal::LuaNum(i.to_be_bytes(), false)),
                    );

                    // Execute the block
                    let return_vals = block.exec(env)?;
                    match return_vals {
                        Some(return_vals) => {
                            if !return_vals.is_empty() {
                                // Return statement
                                env.pop_local_env();
                                return Ok(Some(return_vals));
                            }
                        }
                        None => {
                            // Break statement (exiting loop now so return empty vector)
                            env.pop_local_env();
                            return Ok(Some(vec![]));
                        }
                    };
                    env.pop_local_env();
                    i += step;
                }
            }
            Statement::ForGeneric((names, exp_list, block)) => {
                // Generic for statement must be used with iterator
                // TODO: finish implementing this
                if exp_list.len() != 1 {
                    return Err(ASTExecError(format!(
                        "Generic for loop must use iterator function"
                    )));
                }

                unimplemented!()
            }
            Statement::FunctionDecl((name, par_list, block)) => {
                let captured_variables = block.capture_variables(env);
                env.insert_global(
                    name.clone(),
                    LuaValue::new(LuaVal::Function(LuaFunction {
                        par_list,
                        block,
                        captured_variables,
                    })),
                );
            }
            Statement::LocalFuncDecl((name, par_list, block)) => {
                let captured_variables = block.capture_variables(env);
                env.insert_local(
                    name.clone(),
                    LuaValue::new(LuaVal::Function(LuaFunction {
                        par_list,
                        block,
                        captured_variables,
                    })),
                );
            }
        };

        Ok(Some(vec![]))
    }

    pub fn capture_variables<'a>(&self, env: &Env<'a>) -> Vec<(String, LuaValue<'a>)> {
        let mut captured_vars = vec![];
        match self {
            Statement::Assignment((_, exp_list, _)) => {
                for exp in exp_list {
                    captured_vars.append(&mut exp.capture_variables(env));
                }
            }
            Statement::FunctionCall(func_call) => {
                captured_vars.append(&mut func_call.capture_variables(env));
            }
            Statement::DoBlock(block) => {
                captured_vars.append(&mut block.capture_variables(env));
            }
            Statement::While((exp, block)) => {
                captured_vars.append(&mut exp.capture_variables(env));
                captured_vars.append(&mut block.capture_variables(env));
            }
            Statement::Repeat((block, exp)) => {
                captured_vars.append(&mut block.capture_variables(env));
                captured_vars.append(&mut exp.capture_variables(env));
            }
            Statement::If((exp, block, elifs, else_block)) => {
                captured_vars.append(&mut exp.capture_variables(env));
                captured_vars.append(&mut block.capture_variables(env));
                for (exp, block) in elifs {
                    captured_vars.append(&mut exp.capture_variables(env));
                    captured_vars.append(&mut block.capture_variables(env));
                }
                if let Some(else_block) = else_block {
                    captured_vars.append(&mut else_block.capture_variables(env));
                }
            }
            Statement::ForNum((_, initial, limit, step, block)) => {
                captured_vars.append(&mut initial.capture_variables(env));
                captured_vars.append(&mut limit.capture_variables(env));
                if let Some(step) = step {
                    captured_vars.append(&mut step.capture_variables(env));
                }
                captured_vars.append(&mut block.capture_variables(env));
            }
            Statement::ForGeneric((_, exp_list, block)) => {
                for exp in exp_list {
                    captured_vars.append(&mut exp.capture_variables(env));
                }
                captured_vars.append(&mut block.capture_variables(env));
            }
            Statement::FunctionDecl((_, _, block)) => {
                // Parameters do not capture any variables
                captured_vars.append(&mut block.capture_variables(env));
            }
            Statement::LocalFuncDecl((_, _, block)) => {
                // Parameters do not capture any variables
                captured_vars.append(&mut block.capture_variables(env));
            }
            _ => {}
        };
        captured_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper functions
    fn var_exp(name: &str) -> Expression {
        Expression::PrefixExp(Box::new(PrefixExp::Var(Var::NameVar(name.to_string()))))
    }
    fn integer_exp(n: i64) -> Expression {
        Expression::Numeral(Numeral::Integer(n))
    }
    fn lua_integer<'a>(n: i64) -> LuaValue<'a> {
        LuaValue::new(LuaVal::LuaNum(n.to_be_bytes(), false))
    }
    fn lua_nil<'a>() -> LuaValue<'a> {
        LuaValue::new(LuaVal::LuaNil)
    }
    fn lua_function<'a>(par_list: &'a ParList, block: &'a Block, env: &Env<'a>) -> LuaValue<'a> {
        let captured_variables = block.capture_variables(env);
        LuaValue::new(LuaVal::Function(LuaFunction {
            par_list,
            block,
            captured_variables,
        }))
    }

    #[test]
    fn test_exec_stat_assign() {
        // Test Statement exec method
        let mut env = Env::new();

        // varlist.len > explist.len
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
        let stat = Statement::Assignment((varlist, explist, false));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            *env.get("a").unwrap().0,
            LuaVal::LuaNum(a.to_be_bytes(), false)
        );
        assert_eq!(
            *env.get("b").unwrap().0,
            LuaVal::LuaNum(b.to_be_bytes(), false)
        );

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
        let stat = Statement::Assignment((varlist, explist, false));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            *env.get("a").unwrap().0,
            LuaVal::LuaNum(a.to_be_bytes(), false)
        );
        assert_eq!(
            *env.get("b").unwrap().0,
            LuaVal::LuaNum(b.to_be_bytes(), false)
        );
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
        let stat = Statement::Assignment((varlist, explist, false));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            *env.get("a").unwrap().0,
            LuaVal::LuaNum(a.to_be_bytes(), false)
        );
        assert_eq!(
            *env.get("b").unwrap().0,
            LuaVal::LuaNum(b.to_be_bytes(), false)
        );

        // Local assignment
        env = Env::new();
        let a: i64 = 10;
        let b: i64 = 20;
        let varlist = vec![Var::NameVar("a".to_string()), Var::NameVar("b".to_string())];
        let explist = vec![
            Expression::Numeral(Numeral::Integer(10)),
            Expression::Numeral(Numeral::Integer(20)),
        ];
        let stat = Statement::Assignment((varlist, explist, true));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));

        // Get local variable first
        assert_eq!(
            *env.get_local("a").unwrap().0,
            LuaVal::LuaNum(a.to_be_bytes(), false)
        );
        assert_eq!(
            *env.get_local("b").unwrap().0,
            LuaVal::LuaNum(b.to_be_bytes(), false)
        );
        assert_eq!(env.get_global("a"), None);
    }

    #[test]
    fn test_exec_stat_reassign() {
        // Test Statement exec method
        let mut env = Env::new();

        // reassignment in one line (should not know value of "a" in same line)
        let a: i64 = 10;
        let varlist = vec![
            Var::NameVar("a".to_string()),
            Var::NameVar("b".to_string()),
            Var::NameVar("a".to_string()),
        ];
        let explist = vec![
            Expression::Numeral(Numeral::Integer(30)),
            Expression::PrefixExp(Box::new(PrefixExp::Var(Var::NameVar("a".to_string())))),
            Expression::Numeral(Numeral::Integer(10)),
        ];
        let stat = Statement::Assignment((varlist, explist, false));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            *env.get("a").unwrap().0,
            LuaVal::LuaNum(a.to_be_bytes(), false)
        );
        assert_eq!(*env.get("b").unwrap().0, LuaVal::LuaNil);

        // string reassignment (in assignment for b, should know value of a as 10)
        let a = "testA";
        let b = "testB";
        let varlist = vec![
            Var::NameVar("a".to_string()),
            Var::NameVar("b".to_string()),
            Var::NameVar("a".to_string()),
        ];
        let explist = vec![
            Expression::LiteralString(b.to_string()),
            Expression::PrefixExp(Box::new(PrefixExp::Var(Var::NameVar("a".to_string())))),
            Expression::LiteralString(a.to_string()),
        ];
        let stat = Statement::Assignment((varlist, explist, false));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(*env.get("a").unwrap().0, LuaVal::LuaString(a.to_string()));
        assert_eq!(
            *env.get("b").unwrap().0,
            LuaVal::LuaNum(10_i64.to_be_bytes(), false)
        );
    }

    #[test]
    fn test_exec_stat_visibility() {
        // Test Statement exec method
        let mut env = Env::new();

        // reassignment in one line (should not know value of "a" in same line)
        let a: i64 = 10;
        let stat = Statement::Assignment((
            vec![Var::NameVar("a".to_string())],
            vec![Expression::Numeral(Numeral::Integer(10))],
            false,
        ));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            *env.get("a").unwrap().0,
            LuaVal::LuaNum(a.to_be_bytes(), false)
        );

        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(20))],
                true,
            ))],
            return_stat: Some(vec![Expression::PrefixExp(Box::new(PrefixExp::Var(
                Var::NameVar("a".to_string()),
            )))]),
        };
        let do_block = Statement::DoBlock(block);
        assert_eq!(
            do_block.exec(&mut env),
            Ok(Some(vec![LuaValue::new(LuaVal::LuaNum(
                20_i64.to_be_bytes(),
                false
            ))]))
        );
        // Exited the environment so accessing global "a"
        assert_eq!(
            *env.get("a").unwrap().0,
            LuaVal::LuaNum(a.to_be_bytes(), false)
        );
    }

    #[test]
    fn test_exec_stat_func_call() {
        let mut env = Env::new();
        let a: i64 = 10;
        env.insert_global("a".to_string(), lua_integer(a));

        // Simple function with side effect (a = 10.04)
        let varlist = vec![Var::NameVar("a".to_string())];
        let num: f64 = 10.04;
        let exp_float = Expression::Numeral(Numeral::Float(num));
        let explist = vec![exp_float];
        let stat = Statement::Assignment((varlist, explist, false));
        let return_stat = Some(vec![var_exp("a"), var_exp("a")]);
        let block = Block {
            statements: vec![stat],
            return_stat: return_stat,
        };
        let par_list = ParList(vec![], false);

        // Create function environment
        env.insert_global("f".to_string(), lua_function(&par_list, &block, &env));
        // Function call statement
        let func_prefix = PrefixExp::Var(Var::NameVar("f".to_string()));
        let args = Args::ExpList(vec![]);
        let func_call = FunctionCall::Standard((Box::new(func_prefix), args));
        let func_call_stat = Statement::FunctionCall(func_call);

        // Return "a" defined inside function
        // Returned values are thrown away for statement function call
        assert_eq!(func_call_stat.exec(&mut env), Ok(Some(vec![])));
        // After function call, return global "a"
        assert_eq!(env.get("a"), Some(&lua_integer(a)));
    }

    #[test]
    fn test_exec_stat_if() {
        let mut env = Env::new();
        let condition = Expression::Numeral(Numeral::Integer(0));
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(10))],
                false,
            ))],
            return_stat: None,
        };
        let if_stat = Statement::If((condition, block, vec![], None));
        assert_eq!(if_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(env.get("a"), Some(&lua_integer(10)));
    }

    #[test]
    fn test_exec_stat_else() {
        let mut env = Env::new();
        let condition = Expression::False;
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(10))],
                false,
            ))],
            return_stat: None,
        };
        let else_block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(20))],
                false,
            ))],
            return_stat: None,
        };
        let if_stat = Statement::If((condition, block, vec![], Some(else_block)));
        assert_eq!(if_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(env.get("a"), Some(&lua_integer(20)));
    }

    #[test]
    fn test_exec_stat_elseif() {
        let mut env = Env::new();
        let condition = Expression::False;
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(10))],
                false,
            ))],
            return_stat: None,
        };
        let else_ifs = vec![(
            Expression::True,
            Block {
                statements: vec![Statement::Assignment((
                    vec![Var::NameVar("a".to_string())],
                    vec![Expression::Numeral(Numeral::Integer(20))],
                    false,
                ))],
                return_stat: None,
            },
        )];
        let else_block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(30))],
                false,
            ))],
            return_stat: None,
        };
        let if_stat = Statement::If((condition, block, else_ifs, Some(else_block)));
        assert_eq!(if_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(env.get("a"), Some(&lua_integer(20)));
    }

    #[test]
    fn test_exec_stat_doblock() {
        let mut env = Env::new();
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(10))],
                false,
            ))],
            return_stat: Some(vec![Expression::PrefixExp(Box::new(PrefixExp::Var(
                Var::NameVar("a".to_string()),
            )))]),
        };
        let do_block = Statement::DoBlock(block);
        assert_eq!(do_block.exec(&mut env), Ok(Some(vec![lua_integer(10)])));
        assert_eq!(env.get("a"), Some(&lua_integer(10)));
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
                    false,
                )),
                Statement::Break,
            ],
            return_stat: None,
        };
        let while_stat = Statement::While((condition, block));
        assert_eq!(while_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(env.get("a"), Some(&lua_integer(10)));
        assert_eq!(env.get("a"), Some(&lua_integer(10)));
    }

    #[test]
    fn test_exec_stat_while_return() {
        let mut env = Env::new();
        let condition = Expression::True;
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::Numeral(Numeral::Integer(10))],
                false,
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
        let mut env = Env::new();
        let stat = Statement::Assignment((
            vec![Var::NameVar("a".to_string())],
            vec![Expression::Numeral(Numeral::Integer(10))],
            false,
        ));
        stat.exec(&mut env).unwrap();
        // TODO: change the condition after binary operation is implemented
        let condition = Expression::True;
        let block = Block {
            statements: vec![
                Statement::Assignment((
                    vec![Var::NameVar("a".to_string())],
                    vec![Expression::BinaryOp((
                        Box::new(var_exp("a")),
                        BinOp::Add,
                        Box::new(var_exp("a")),
                    ))],
                    false,
                )),
                Statement::Break,
            ],
            return_stat: None,
        };

        let while_stat = Statement::While((condition, block));
        assert_eq!(while_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(env.get("a"), Some(&lua_integer(20)));
    }

    #[test]
    fn test_exec_stat_for_num() {
        // TODO: increment few times when binary expression is implemented
        let mut env = Env::new();
        let for_stat = Statement::ForNum((
            "i".to_string(),
            Expression::Numeral(Numeral::Integer(0)),
            Expression::Numeral(Numeral::Integer(2)),
            None,
            Block {
                statements: vec![Statement::Assignment((
                    vec![Var::NameVar("a".to_string())],
                    vec![Expression::Numeral(Numeral::Integer(20))],
                    false,
                ))],
                return_stat: None,
            },
        ));
        assert_eq!(for_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(env.get("a"), Some(&lua_integer(20)));
    }

    #[test]
    fn test_exec_stat_for_num_break() {
        let mut env = Env::new();
        let for_stat = Statement::ForNum((
            "i".to_string(),
            Expression::Numeral(Numeral::Integer(2)),
            Expression::Numeral(Numeral::Integer(0)),
            Some(Expression::Numeral(Numeral::Integer(-1))),
            Block {
                statements: vec![
                    Statement::Break,
                    Statement::Assignment((
                        vec![Var::NameVar("a".to_string())],
                        vec![Expression::Numeral(Numeral::Integer(20))],
                        false,
                    )),
                ],
                return_stat: None,
            },
        ));
        assert_eq!(for_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(env.get("a"), None);
    }

    #[test]
    fn test_exec_stat_for_num_return() {
        let mut env = Env::new();
        let for_stat = Statement::ForNum((
            "i".to_string(),
            Expression::Numeral(Numeral::Integer(2)),
            Expression::Numeral(Numeral::Integer(0)),
            Some(Expression::Numeral(Numeral::Integer(-1))),
            Block {
                statements: vec![Statement::Assignment((
                    vec![Var::NameVar("a".to_string())],
                    vec![Expression::Numeral(Numeral::Integer(20))],
                    false,
                ))],
                return_stat: Some(vec![var_exp("a")]),
            },
        ));
        assert_eq!(for_stat.exec(&mut env), Ok(Some(vec![lua_integer(20)])));
    }

    #[test]
    fn test_exec_stat_for_num_step_zero() {
        let mut env = Env::new();
        let for_stat = Statement::ForNum((
            "i".to_string(),
            Expression::Numeral(Numeral::Integer(2)),
            Expression::Numeral(Numeral::Integer(0)),
            Some(Expression::Numeral(Numeral::Integer(0))),
            Block {
                statements: vec![Statement::Assignment((
                    vec![Var::NameVar("a".to_string())],
                    vec![Expression::Numeral(Numeral::Integer(20))],
                    false,
                ))],
                return_stat: None,
            },
        ));
        assert_eq!(
            for_stat.exec(&mut env),
            Err(ASTExecError(format!("Step value in for loop cannot be 0")))
        );
    }

    #[test]
    fn test_exec_stat_func_decl() {
        let mut env = Env::new();
        let func_decl = Statement::FunctionDecl((
            "f".to_string(),
            ParList(vec![String::from("a"), String::from("b")], false),
            Block {
                statements: vec![],
                return_stat: Some(vec![Expression::BinaryOp((
                    Box::new(var_exp("a")),
                    BinOp::Add,
                    Box::new(var_exp("b")),
                ))]),
            },
        ));
        assert_eq!(func_decl.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            env.get_global("f"),
            Some(&lua_function(
                &ParList(vec![String::from("a"), String::from("b")], false),
                &Block {
                    statements: vec![],
                    return_stat: Some(vec![Expression::BinaryOp((
                        Box::new(var_exp("a")),
                        BinOp::Add,
                        Box::new(var_exp("b"))
                    ))])
                },
                &env
            ))
        );
    }

    #[test]
    fn test_exec_stat_func_decl_local() {
        let mut env = Env::new();
        let func_decl = Statement::LocalFuncDecl((
            "f".to_string(),
            ParList(vec![String::from("a"), String::from("b")], false),
            Block {
                statements: vec![],
                return_stat: Some(vec![Expression::BinaryOp((
                    Box::new(var_exp("a")),
                    BinOp::Add,
                    Box::new(var_exp("b")),
                ))]),
            },
        ));
        assert_eq!(func_decl.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(
            env.get_local("f"),
            Some(&lua_function(
                &ParList(vec![String::from("a"), String::from("b")], false),
                &Block {
                    statements: vec![],
                    return_stat: Some(vec![Expression::BinaryOp((
                        Box::new(var_exp("a")),
                        BinOp::Add,
                        Box::new(var_exp("b"))
                    ))]),
                },
                &env
            ))
        );
    }

    #[test]
    fn test_exec_func_capture_variables() {
        let mut env = Env::new();

        let stat = Statement::Assignment((
            vec![Var::NameVar("c".to_string())],
            vec![Expression::BinaryOp((
                Box::new(var_exp("a")),
                BinOp::Add,
                Box::new(var_exp("b")),
            ))],
            false,
        ));
        let func_decl = Statement::FunctionDecl((
            "f".to_string(),
            ParList(vec![], false),
            Block {
                statements: vec![stat],
                return_stat: Some(vec![var_exp("c")]),
            },
        ));
        let assignments = Statement::Assignment((
            vec![Var::NameVar("a".to_string()), Var::NameVar("b".to_string())],
            vec![integer_exp(10), integer_exp(20)],
            true,
        ));
        let doblock = Statement::DoBlock(Block {
            statements: vec![assignments, func_decl],
            return_stat: None,
        });

        // f(100) executes a = 30, b = 20, return test
        assert_eq!(doblock.exec(&mut env), Ok(Some(vec![])));
        // Local variables can't be accessed after exiting scope
        assert_eq!(env.get("a"), None);
        assert_eq!(env.get("b"), None);

        // But function (closure) can access them
        let args = Args::ExpList(vec![]);
        let func_call = FunctionCall::Standard((
            Box::new(PrefixExp::Var(Var::NameVar("f".to_string()))),
            args,
        ));
        let exp = PrefixExp::FunctionCall(func_call);
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(30))); // a + b = 10 + 20
    }

    #[test]
    fn test_capture_variables() {
        let mut env = Env::new();
        env.insert_local("a".to_string(), lua_integer(1));
        env.insert_local("b".to_string(), lua_integer(2));
        env.insert_local("c".to_string(), lua_integer(3));
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("d".to_string())],
                vec![Expression::BinaryOp((
                    Box::new(var_exp("a")),
                    BinOp::Add,
                    Box::new(var_exp("b")),
                ))],
                false,
            ))],
            return_stat: Some(vec![var_exp("c"), var_exp("d")]),
        };

        assert_eq!(
            block.capture_variables(&env),
            vec![
                ("a".to_string(), lua_integer(1)),
                ("b".to_string(), lua_integer(2)),
                ("c".to_string(), lua_integer(3)),
                ("d".to_string(), lua_nil())
            ]
        );
        assert_eq!(
            block.exec(&mut env),
            Ok(Some(vec![lua_integer(3), lua_integer(3)]))
        );
    }
}
