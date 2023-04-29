use crate::ast::*;
use crate::interpreter::environment::Env;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::{cell::RefCell, rc::Rc};
use std::collections::HashMap;

use self::environment::LocalEnv;

pub mod environment;
pub mod expression;
pub mod statement;

#[derive(Debug, PartialEq)]
pub enum LuaVal<'a>{
    // May not need RefCell
    LuaTable(RefCell<LuaTable<'a>>),
    LuaNil,
    LuaBool(bool),
    LuaNum([u8; 8], bool), // numerals as an array of 8 bytes, bool for is_float
    LuaString(String),
    Function(LuaFunction<'a>),
    Print,
    TestPrint(Rc<RefCell<Vec<String>>>),
    Read,
}

// Lua function captures environment in function call
#[derive(Debug, PartialEq)]
pub struct LuaFunction<'a> {
    par_list: &'a ParList,
    block: &'a Block,
    captured_env: LocalEnv<'a>,
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

    pub fn is_zero(&self) -> bool {
        match &*self.0 {
            LuaVal::LuaNum(num, is_float) => {
                if *is_float {
                    let num = f64::from_be_bytes(*num);
                    num == 0.0
                } else {
                    let num = i64::from_be_bytes(*num);
                    num == 0
                }
            }
            _ => false,
        }
    }

    pub fn is_positive(&self) -> bool {
        match &*self.0 {
            LuaVal::LuaNum(num, is_float) => {
                if *is_float {
                    let num = f64::from_be_bytes(*num);
                    num > 0.0
                } else {
                    let num = i64::from_be_bytes(*num);
                    num > 0
                }
            }
            _ => false,
        }
    }

    pub fn is_negative(&self) -> bool {
        match &*self.0 {
            LuaVal::LuaNum(num, is_float) => {
                if *is_float {
                    let num = f64::from_be_bytes(*num);
                    num < 0.0
                } else {
                    let num = i64::from_be_bytes(*num);
                    num < 0
                }
            }
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match &*self.0 {
            LuaVal::LuaNil => true,
            _ => false,
        }
    }

    pub fn is_greater_or_equal(&self, num: i64) -> Result<bool, ASTExecError> {
        match &*self.0 {
            LuaVal::LuaNum(n, is_float) => {
                if *is_float {
                    let n = f64::from_be_bytes(*n);
                    if n.floor() as i64 >= num {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    let n = i64::from_be_bytes(*n);
                    Ok(n >= num)
                }
            }
            _ => Err(ASTExecError(format!(
                "Cannot compare values (types cannot be compared)"
            ))),
        }
    }

    pub fn is_less_or_equal(&self, num: i64) -> Result<bool, ASTExecError> {
        match &*self.0 {
            LuaVal::LuaNum(n, is_float) => {
                if *is_float {
                    let n = f64::from_be_bytes(*n);
                    if n.ceil() as i64 <= num {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    let n = i64::from_be_bytes(*n);
                    Ok(n <= num)
                }
            }
            _ => Err(ASTExecError(format!(
                "Cannot compare values (types cannot be compared)"
            ))),
        }
    }

    pub fn negate_bool(self) -> Result<LuaValue<'a>, ASTExecError> {
        match self.0.as_ref() {
            LuaVal::LuaBool(b) => Ok(LuaValue::new(LuaVal::LuaBool(!b))),
            _ => Err(ASTExecError(format!(
                "Cannot negate value (only boolean can be negated)"
            ))),
        }
    }

    pub fn into_int(self) -> Result<i64, ASTExecError> {
        match self.0.as_ref() {
            LuaVal::LuaNum(n, is_float) => {
                if *is_float {
                    let n = f64::from_be_bytes(*n);
                    if n.floor() == n.ceil() {
                        Ok(n.floor() as i64)
                    } else {
                        Err(ASTExecError(format!(
                            "Cannot convert float that does not have exact integer value to integer"
                        )))
                    }
                } else {
                    Ok(i64::from_be_bytes(*n))
                }
            }
            _ => Err(ASTExecError(format!(
                "Cannot convert value to integer (types cannot be converted)"
            ))),
        }
    }

    pub fn into_string(self) -> Result<String, ASTExecError> {
        match self.0.as_ref() {
            LuaVal::LuaNum(n, is_float) => {
                if *is_float {
                    let n = f64::from_be_bytes(*n);
                    if n.floor() != n.ceil() {
                        Ok(n.to_string())
                    } else {
                        // If n = 23.0, make it print as 23.0 instead of 23
                        Ok(format!("{:.1}", n))
                    }
                } else {
                    Ok(i64::from_be_bytes(*n).to_string())
                }
            }
            LuaVal::LuaString(s) => Ok(s.clone()),
            _ => Err(ASTExecError(format!(
                "Cannot convert value to String (types cannot be converted)"
            ))),
        }
    }

    fn extract_first_return_val(return_vals: Vec<LuaValue>) -> LuaValue {
        if return_vals.is_empty() {
            // If no return values, return nil
            LuaValue::new(LuaVal::LuaNil)
        } else {
            return_vals[0].clone()
        }
    }
}

impl<'a> Display for LuaValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &*self.0 {
            LuaVal::LuaNil => write!(f, "nil"),
            LuaVal::LuaBool(b) => write!(f, "{b}"),
            LuaVal::LuaNum(n, is_float) => {
                if *is_float {
                    let n = f64::from_be_bytes(*n);
                    if n.floor() != n.ceil() {
                        write!(f, "{n}")
                    } else {
                        // If n = 23.0, make it print as 23.0 instead of 23
                        write!(f, "{:.1}", n)
                    }
                } else {
                    write!(f, "{}", i64::from_be_bytes(*n))
                }
            }
            LuaVal::LuaString(s) => write!(f, "{}", s),
            LuaVal::LuaTable(t) => write!(f, "{:p}", t),
            // Display function as reference
            LuaVal::Function(func) => write!(f, "{:p}", func),
            LuaVal::Print => write!(f, "print"),
            LuaVal::TestPrint(_) => write!(f, "print"),
            LuaVal::Read => write!(f, "read"),
        }
    }
}

// TODO: use hashmap representation since key can be only string or number
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
enum TableKey {
    String(String),
    Number(i64),
}

// TODO: IMPORTANT - make sure to update hashmap inside the RefCell when the table is being mutated
// Instead of overwriting the entire Rc
#[derive(Debug, PartialEq, Clone)]
pub struct LuaTable<'a>(Rc<RefCell<HashMap<TableKey, LuaValue<'a>>>>);
impl<'a> LuaTable<'a> {
    pub fn new() -> Self {
        LuaTable(Rc::new(RefCell::new(HashMap::new())))
    }

    // TODO: implement table methods
    // pub fn insert(&self, key: LuaValue<'a>, val: LuaValue<'a>) {
    //     self.0.borrow_mut().push((key, val));
    // }

    // pub fn get(&self, key: &LuaValue<'a>) -> Option<LuaValue<'a>> {
    //     for (k, v) in self.0.borrow().iter() {
    //         if k == key {
    //             return Some(v.clone());
    //         }
    //     }
    //     None
    // }

    // pub fn remove(&self, key: &LuaValue<'a>) {
    //     let mut table = self.0.borrow_mut();
    //     let mut i = 0;
    //     while i < table.len() {
    //         if table[i].0 == *key {
    //             table.remove(i);
    //             break;
    //         }
    //         i += 1;
    //     }
    // }
}

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

    // Used for repeat-until loops (need to refer to local variables inside the loop)
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
            } else if let Some(return_vals) = return_vals {
                if !return_vals.is_empty() {
                    // Return statement inside statement list
                    return Ok(Some(return_vals));
                }
            }
        }

        // Optional return statement
        let explist = match &self.return_stat {
            Some(explist) => explist,
            // Returning empty vector means there was no return statement
            None => return Ok(Some(vec![])),
        };

        let mut return_vals = Vec::with_capacity(explist.len());
        let mut i = 0;
        for exp in explist.iter() {
            // For each expression, only the first return value is used,
            // but last expression can use multiple return values
            if i == explist.len() - 1 {
                return_vals.append(&mut exp.eval(env).unwrap());
                break;
            }
            return_vals.push(LuaValue::extract_first_return_val(exp.eval(env).unwrap()));
            i += 1;
        }
        Ok(Some(return_vals))
    }

    // Find captured variables in the block
    // Dead code
    // fn capture_variables<'a>(&self, env: &Env<'a>) -> Vec<(String, LuaValue<'a>)> {
    //     // CONTINUE: not capturing variables correctly
    //     let mut captured_vars = vec![];
    //     for statement in &self.statements {
    //         captured_vars.append(&mut statement.capture_variables(env));
    //     }
    //     if let Some(return_stat) = &self.return_stat {
    //         for exp in return_stat {
    //             captured_vars.append(&mut exp.capture_variables(env));
    //         }
    //     }
    //     captured_vars
    // }
}

#[derive(Debug, PartialEq)]
pub struct ASTExecError(String);
impl ASTExecError {
    pub fn new(msg: &str) -> Self {
        ASTExecError(msg.to_string())
    }
}
impl Display for ASTExecError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
