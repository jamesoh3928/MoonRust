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
                fn insert_to_env<'a>(
                    var: &'a Var,
                    val: &LuaValue<'a>,
                    env: &mut Env<'a>,
                    is_local: &bool,
                ) -> Result<(), ASTExecError> {
                    match var {
                        Var::NameVar(name) => {
                            // Insert into environment
                            if *is_local {
                                // With local keyword, always insert new local variable or overwrite existing
                                env.insert_local(name.clone(), val.clone());
                            } else {
                                if env.get_local(name).is_some() {
                                    // Update local variable
                                    env.update_local(name.clone(), val.clone());
                                } else {
                                    // Update or insert global variable
                                    env.insert_global(name.clone(), val.clone());
                                }
                            }
                        }
                        // TODO: implement after table (make sure you don't overwrite table, you have to mutate the table)
                        Var::BracketVar((prefixexp, exp)) => {
                            let prefixexp =
                                LuaValue::extract_first_return_val(prefixexp.eval(env)?);
                            match prefixexp.0.as_ref() {
                                LuaVal::LuaTable(table) => {
                                    let key = LuaValue::extract_first_return_val(exp.eval(env)?);
                                    table.insert(key, val.clone())?;
                                }
                                _ => {
                                    return Err(ASTExecError(format!(
                                        "attempt to index a non-table value '{prefixexp}'"
                                    )))
                                }
                            }
                        }
                        Var::DotVar((prefixexp, field)) => {
                            // TODO: Factor this out into its own function
                            let prefixexp =
                                LuaValue::extract_first_return_val(prefixexp.eval(env)?);
                            match prefixexp.0.as_ref() {
                                LuaVal::LuaTable(table) => {
                                    table.insert_ident(field.clone(), val.clone());
                                }
                                _ => {
                                    return Err(ASTExecError(format!(
                                        "attempt to index a non-table value '{prefixexp}'"
                                    )))
                                }
                            }
                        }
                    }
                    Ok(())
                }

                // If there are more values than needed, the excess values are thrown away.
                let mut vallist = Vec::with_capacity(varlist.len());
                let mut i = 0; // index for the current variable
                while i < varlist.len() {
                    let return_vals = explist[i].eval(env)?;
                    let num = if i == explist.len() - 1 {
                        // Extract return values to match number of variables to be matched or all return values
                        // We are doing this because we can assignm multipe value if last expression is function call
                        varlist.len() - i
                    } else {
                        // extract first return value
                        1
                    };
                    for j in 0..num {
                        if j > return_vals.len() - 1 {
                            // More variables than total return values: insert nil
                            vallist.push(LuaValue::new(LuaVal::LuaNil));
                        } else {
                            vallist.push(return_vals[j].clone());
                        };
                        i += 1;
                    }
                }
                // Insert into the environment
                for i in 0..varlist.len() {
                    insert_to_env(&varlist[i], &vallist[i], env, is_local)?;
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
                while LuaValue::extract_first_return_val(exp.eval(env)?).is_true() {
                    let return_vals = block.exec(env)?;
                    match return_vals {
                        Some(return_vals) => {
                            if !return_vals.is_empty() {
                                // Return statement
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
                    if LuaValue::extract_first_return_val(exp.eval(env)?).is_true() {
                        break;
                    }
                    // Pop local environment before next iteration
                    env.pop_local_env();
                }
            }
            Statement::If((exp, block, elseifs, elseblock)) => {
                let condition = LuaValue::extract_first_return_val(exp.eval(env)?);
                if condition.is_true() {
                    return block.exec(env);
                } else {
                    // Do elseifs
                    for (exp, block) in elseifs {
                        let condition = LuaValue::extract_first_return_val(exp.eval(env)?);
                        if condition.is_true() {
                            return block.exec(env);
                        }
                    }
                    if let Some(elseblock) = elseblock {
                        return elseblock.exec(env);
                    }
                }
            }
            Statement::ForNum((name, exp1, exp2, exp3, block)) => {
                let initial = match LuaValue::extract_first_return_val(exp1.eval(env)?) {
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
                let limit = LuaValue::extract_first_return_val(exp2.eval(env)?);
                let step = match exp3 {
                    Some(exp) => match LuaValue::extract_first_return_val(exp.eval(env)?) {
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
                // TODO: finish implementing this if there is extra time
                if exp_list.len() != 1 {
                    return Err(ASTExecError(format!(
                        "Generic for loop must use iterator function"
                    )));
                }

                unimplemented!()
            }
            Statement::FunctionDecl((name, par_list, block)) => {
                let captured_env = env.get_local_env().capture_env();
                env.extend_local_without_scope();
                env.insert_global(
                    name.clone(),
                    LuaValue::new(LuaVal::Function(LuaFunction {
                        par_list,
                        block,
                        captured_env,
                    })),
                );
            }
            Statement::LocalFuncDecl((name, par_list, block)) => {
                let captured_env = env.get_local_env().capture_env();
                env.extend_local_without_scope();
                // TODO: changed for testing
                env.insert_local(
                    name.clone(),
                    LuaValue::new(LuaVal::Function(LuaFunction {
                        par_list,
                        block,
                        captured_env,
                    })),
                );
            }
        };

        Ok(Some(vec![]))
    }

    // pub fn capture_variables<'a>(&self, env: &Env<'a>) -> Vec<(String, LuaValue<'a>)> {
    //     let mut captured_vars = vec![];
    //     match self {
    //         Statement::Assignment((_, exp_list, _)) => {
    //             for exp in exp_list {
    //                 captured_vars.append(&mut exp.capture_variables(env));
    //             }
    //         }
    //         Statement::FunctionCall(func_call) => {
    //             captured_vars.append(&mut func_call.capture_variables(env));
    //         }
    //         Statement::DoBlock(block) => {
    //             captured_vars.append(&mut block.capture_variables(env));
    //         }
    //         Statement::While((exp, block)) => {
    //             captured_vars.append(&mut exp.capture_variables(env));
    //             captured_vars.append(&mut block.capture_variables(env));
    //         }
    //         Statement::Repeat((block, exp)) => {
    //             captured_vars.append(&mut block.capture_variables(env));
    //             captured_vars.append(&mut exp.capture_variables(env));
    //         }
    //         Statement::If((exp, block, elifs, else_block)) => {
    //             captured_vars.append(&mut exp.capture_variables(env));
    //             captured_vars.append(&mut block.capture_variables(env));
    //             for (exp, block) in elifs {
    //                 captured_vars.append(&mut exp.capture_variables(env));
    //                 captured_vars.append(&mut block.capture_variables(env));
    //             }
    //             if let Some(else_block) = else_block {
    //                 captured_vars.append(&mut else_block.capture_variables(env));
    //             }
    //         }
    //         Statement::ForNum((_, initial, limit, step, block)) => {
    //             captured_vars.append(&mut initial.capture_variables(env));
    //             captured_vars.append(&mut limit.capture_variables(env));
    //             if let Some(step) = step {
    //                 captured_vars.append(&mut step.capture_variables(env));
    //             }
    //             captured_vars.append(&mut block.capture_variables(env));
    //         }
    //         Statement::ForGeneric((_, exp_list, block)) => {
    //             for exp in exp_list {
    //                 captured_vars.append(&mut exp.capture_variables(env));
    //             }
    //             captured_vars.append(&mut block.capture_variables(env));
    //         }
    //         Statement::FunctionDecl((_, _, block)) => {
    //             // Parameters do not capture any variables
    //             captured_vars.append(&mut block.capture_variables(env));
    //         }
    //         Statement::LocalFuncDecl((_, _, block)) => {
    //             // Parameters do not capture any variables
    //             captured_vars.append(&mut block.capture_variables(env));
    //         }
    //         _ => {}
    //     };
    //     captured_vars
    // }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap};

    use crate::interpreter::{LuaTable, TableKey};

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
    fn lua_float<'a>(n: f64) -> LuaValue<'a> {
        LuaValue::new(LuaVal::LuaNum(n.to_be_bytes(), true))
    }
    fn lua_function<'a>(par_list: &'a ParList, block: &'a Block, env: &Env<'a>) -> LuaValue<'a> {
        let captured_env = env.get_local_env().capture_env();
        LuaValue::new(LuaVal::Function(LuaFunction {
            par_list,
            block,
            captured_env,
        }))
    }
    fn lua_table<'a>(hmap: HashMap<TableKey, LuaValue<'a>>) -> Vec<LuaValue<'a>> {
        vec![LuaValue::new(LuaVal::LuaTable(LuaTable(RefCell::new(
            hmap,
        ))))]
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
    fn test_exec_stat_table_assign() {
        let mut env = Env::new();

        let table = LuaValue::extract_first_return_val(lua_table(HashMap::from([(
            TableKey::String(String::from("x")),
            LuaValue::new(LuaVal::LuaNum(i64::to_be_bytes(999), false)),
        )])));

        env.insert_global(String::from("my_table"), table);

        let stat = Statement::Assignment((
            vec![Var::DotVar((
                Box::new(PrefixExp::Var(Var::NameVar(String::from("my_table")))),
                String::from("x"),
            ))],
            vec![Expression::LiteralString(String::from("new value!"))],
            false,
        ));
        assert_eq!(stat.exec(&mut env), Ok(Some(vec![])));

        let expected_table = LuaValue::extract_first_return_val(lua_table(HashMap::from([(
            TableKey::String(String::from("x")),
            LuaValue::new(LuaVal::LuaString(String::from("new value!"))),
        )])));
        let actual_table = &*env.get("my_table").unwrap().0;
        assert_eq!(actual_table, &*expected_table.0)
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
        // After function call, return global "a" which is updated to float value
        assert_eq!(env.get("a"), Some(&lua_float(10.04)));
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
        let mut env = Env::new();
        let stat = Statement::Assignment((
            vec![Var::NameVar("a".to_string())],
            vec![Expression::Numeral(Numeral::Integer(10))],
            false,
        ));
        stat.exec(&mut env).unwrap();
        let condition = Expression::BinaryOp((
            Box::new(var_exp("a")),
            BinOp::LessEq,
            Box::new(Expression::Numeral(Numeral::Integer(15))),
        ));
        let block = Block {
            statements: vec![Statement::Assignment((
                vec![Var::NameVar("a".to_string())],
                vec![Expression::BinaryOp((
                    Box::new(var_exp("a")),
                    BinOp::Add,
                    Box::new(Expression::Numeral(Numeral::Integer(2))),
                ))],
                false,
            ))],
            return_stat: None,
        };

        let while_stat = Statement::While((condition, block));
        assert_eq!(while_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(env.get("a"), Some(&lua_integer(16)));
    }

    #[test]
    fn test_exec_stat_for_num() {
        let mut env = Env::new();
        let stat = Statement::Assignment((
            vec![Var::NameVar("a".to_string())],
            vec![Expression::Numeral(Numeral::Integer(10))],
            false,
        ));
        stat.exec(&mut env).unwrap();
        let for_stat = Statement::ForNum((
            "i".to_string(),
            Expression::Numeral(Numeral::Integer(1)),
            Expression::Numeral(Numeral::Integer(5)),
            None,
            Block {
                statements: vec![Statement::Assignment((
                    vec![Var::NameVar("a".to_string())],
                    vec![Expression::BinaryOp((
                        Box::new(var_exp("a")),
                        BinOp::Add,
                        Box::new(var_exp("i")),
                    ))],
                    false,
                ))],
                return_stat: None,
            },
        ));
        assert_eq!(for_stat.exec(&mut env), Ok(Some(vec![])));
        assert_eq!(env.get("a"), Some(&lua_integer(25)));
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
        let par_list = ParList(vec![String::from("a"), String::from("b")], false);
        let block = Block {
            statements: vec![],
            return_stat: Some(vec![Expression::BinaryOp((
                Box::new(var_exp("a")),
                BinOp::Add,
                Box::new(var_exp("b")),
            ))]),
        };
        let expected_func = lua_function(&par_list, &block, &env);
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
        assert_eq!(env.get_global("f"), Some(&expected_func));
    }

    #[test]
    fn test_exec_stat_func_decl_local() {
        let mut env = Env::new();
        let par_list = ParList(vec![String::from("a"), String::from("b")], false);
        let block = Block {
            statements: vec![],
            return_stat: Some(vec![Expression::BinaryOp((
                Box::new(var_exp("a")),
                BinOp::Add,
                Box::new(var_exp("b")),
            ))]),
        };
        let expected_func = lua_function(&par_list, &block, &env);
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
        assert_eq!(env.get_local("f"), Some(&expected_func));
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
        assert_eq!(exp.eval(&mut env), Ok(vec![lua_integer(30)])); // a + b = 10 + 20
    }

    // TODO: delete all capture variables code when ensured that capture environment works
    // #[test]
    // fn test_capture_variables() {
    //     let mut env = Env::new();
    //     env.insert_local("a".to_string(), lua_integer(1));
    //     env.insert_local("b".to_string(), lua_integer(2));
    //     env.insert_local("c".to_string(), lua_integer(3));
    //     let block = Block {
    //         statements: vec![Statement::Assignment((
    //             vec![Var::NameVar("d".to_string())],
    //             vec![Expression::BinaryOp((
    //                 Box::new(var_exp("a")),
    //                 BinOp::Add,
    //                 Box::new(var_exp("b")),
    //             ))],
    //             false,
    //         ))],
    //         return_stat: Some(vec![var_exp("c"), var_exp("d")]),
    //     };

    //     assert_eq!(
    //         block.capture_variables(&env),
    //         vec![
    //             ("a".to_string(), lua_integer(1)),
    //             ("b".to_string(), lua_integer(2)),
    //             ("c".to_string(), lua_integer(3)),
    //         ]
    //     );
    //     assert_eq!(
    //         block.exec(&mut env),
    //         Ok(Some(vec![lua_integer(3), lua_integer(3)]))
    //     );
    // }
}
