use crate::ast::*;
use crate::interpreter::environment::Env;
use crate::interpreter::ASTExecError;
use crate::interpreter::LuaFunction;
use crate::interpreter::LuaTable;
use crate::interpreter::LuaVal;
use crate::interpreter::LuaValue;
use std::io;
use std::rc::Rc;

enum IntFloatBool {
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Expression {
    pub fn eval<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<Vec<LuaValue<'a>>, ASTExecError> {
        let val = match self {
            Expression::Nil => vec![LuaValue::new(LuaVal::LuaNil)],
            Expression::False => vec![LuaValue::new(LuaVal::LuaBool(false))],
            Expression::True => vec![LuaValue::new(LuaVal::LuaBool(true))],
            Expression::Numeral(n) => match n {
                Numeral::Integer(i) => vec![LuaValue::new(LuaVal::LuaNum(i.to_be_bytes(), false))],
                Numeral::Float(f) => vec![LuaValue::new(LuaVal::LuaNum(f.to_be_bytes(), true))],
            },
            Expression::LiteralString(s) => vec![LuaValue::new(LuaVal::LuaString(s.clone()))],
            // TODO: DotDotDot? maybe skip it for now
            Expression::DotDotDot => unimplemented!(),
            Expression::FunctionDef((par_list, block)) => {
                let captured_variables = block.capture_variables(env);
                vec![LuaValue::new(LuaVal::Function(LuaFunction {
                    par_list,
                    block,
                    captured_variables,
                }))]
            }
            Expression::PrefixExp(prefixexp) => prefixexp.eval(env)?,
            Expression::TableConstructor(fields) => {
                // TODO
                let table = LuaTable::new(fields.len());
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
                vec![Expression::eval_binary_exp(op, left, right, env)?]
            }
            Expression::UnaryOp((op, exp)) => vec![Expression::eval_unary_exp(op, exp, env)?],
        };
        Ok(val)
    }

    pub fn capture_variables<'a>(&self, env: &Env<'a>) -> Vec<(String, LuaValue<'a>)> {
        match self {
            Expression::FunctionDef((_, block)) => block.capture_variables(env),
            Expression::PrefixExp(prefixexp) => prefixexp.capture_variables(env),
            Expression::TableConstructor(_) => unimplemented!(),
            Expression::BinaryOp((left, _, right)) => {
                let mut vars = left.capture_variables(env);
                vars.append(&mut right.capture_variables(env));
                vars
            }
            Expression::UnaryOp((_, exp)) => exp.capture_variables(env),
            _ => vec![],
        }
    }

    pub fn eval_unary_exp<'a>(
        op: &UnOp,
        exp: &'a Box<Expression>,
        env: &mut Env<'a>,
    ) -> Result<LuaValue<'a>, ASTExecError> {
        match op {
            UnOp::Negate => {
                match &*LuaValue::extract_first_return_val(exp.eval(env)?)
                    .0
                    .borrow()
                {
                    LuaVal::LuaNum(bytes, is_float) => {
                        if !is_float {
                            // Integer
                            let i = i64::from_be_bytes(*bytes);
                            Ok(LuaValue::new(LuaVal::LuaNum((-i).to_be_bytes(), false)))
                        } else {
                            // Float
                            let f = f64::from_be_bytes(*bytes);
                            Ok(LuaValue::new(LuaVal::LuaNum((-f).to_be_bytes(), true)))
                        }
                    }
                    _ => {
                        return Err(ASTExecError(format!(
                            "Cannot negate values that are not numbers"
                        )));
                    }
                }
            }
            UnOp::LogicalNot => {
                if LuaValue::extract_first_return_val(exp.eval(env)?).is_true() {
                    // Negate the true
                    Ok(LuaValue::new(LuaVal::LuaBool(false)))
                } else {
                    // Negate the false
                    Ok(LuaValue::new(LuaVal::LuaBool(true)))
                }
            }
            UnOp::Length => {
                match &*LuaValue::extract_first_return_val(exp.eval(env)?)
                    .0
                    .borrow()
                {
                    LuaVal::LuaString(s) => {
                        // length of a string is its number of bytes
                        Ok(LuaValue::new(LuaVal::LuaNum(
                            (s.len() as i64).to_be_bytes(),
                            false,
                        )))
                    }
                    LuaVal::LuaTable(table) => {
                        // TODO: implement after table
                        // The length operator applied on a table returns a border in that table (check reference manual)
                        unimplemented!()
                    }
                    _ => {
                        return Err(ASTExecError(format!(
                            "Cannot get length of value that is not a string or table"
                        )));
                    }
                }
            }
            UnOp::BitNot => {
                // operate on all bits of those integers, and result in an integer.
                let val = LuaValue::extract_first_return_val(exp.eval(env)?);
                let val = val.into_int()?;
                Ok(LuaValue::new(LuaVal::LuaNum((!val).to_be_bytes(), false)))
            }
        }
    }

    pub fn eval_binary_exp<'a>(
        op: &BinOp,
        left: &'a Box<Expression>,
        right: &'a Box<Expression>,
        env: &mut Env<'a>,
    ) -> Result<LuaValue<'a>, ASTExecError> {
        fn execute_arithmetic<'a, F1, F2>(
            exec_ints: F1,
            exec_floats: F2,
            left: &LuaVal,
            right: &LuaVal,
        ) -> Result<LuaValue<'a>, ASTExecError>
        where
            F1: FnOnce(i64, i64) -> IntFloatBool,
            F2: FnOnce(f64, f64) -> IntFloatBool,
        {
            // If both are integers, the operation is performed over integers and the result is an integer.
            // If both are numbers, then they are converted to floats
            let result = match (left, right) {
                (LuaVal::LuaNum(bytes1, is_float1), LuaVal::LuaNum(bytes2, is_float2)) => {
                    if *is_float1 && *is_float2 {
                        // Both are float
                        let f1 = f64::from_be_bytes(*bytes1);
                        let f2 = f64::from_be_bytes(*bytes2);
                        exec_floats(f1, f2)
                    } else if *is_float1 {
                        // Left is float, right is integer
                        let f1 = f64::from_be_bytes(*bytes1);
                        let i2 = i64::from_be_bytes(*bytes2);
                        exec_floats(f1, i2 as f64)
                    } else if *is_float2 {
                        // Right is float, left is integer
                        let i1 = i64::from_be_bytes(*bytes1);
                        let f2 = f64::from_be_bytes(*bytes2);
                        exec_floats(i1 as f64, f2)
                    } else {
                        // Both are integers
                        let i1 = i64::from_be_bytes(*bytes1);
                        let i2 = i64::from_be_bytes(*bytes2);
                        exec_ints(i1, i2)
                    }
                }
                // TODO: string coercion to numbers (maybe skip for now)
                _ => {
                    return Err(ASTExecError(format!(
                        "Cannot execute opration on values that are not numbers"
                    )));
                }
            };

            match result {
                IntFloatBool::Int(i) => Ok(LuaValue::new(LuaVal::LuaNum(i.to_be_bytes(), false))),
                IntFloatBool::Float(f) => Ok(LuaValue::new(LuaVal::LuaNum(f.to_be_bytes(), true))),
                IntFloatBool::Bool(bool) => Ok(LuaValue::new(LuaVal::LuaBool(bool))),
            }
        }

        fn equal<'a>(
            left: LuaValue<'a>,
            right: LuaValue<'a>,
        ) -> Result<LuaValue<'a>, ASTExecError> {
            match (&*left.0.borrow(), &*right.0.borrow()) {
                (LuaVal::LuaNil, LuaVal::LuaNil) => Ok(LuaValue::new(LuaVal::LuaBool(true))),
                // If number, check if they are equal based on mathematical values
                (LuaVal::LuaNum(_, _), LuaVal::LuaNum(_, _)) => execute_arithmetic(
                    |i1, i2| IntFloatBool::Bool(i1 == i2),
                    |f1, f2| IntFloatBool::Bool(f1 == f2),
                    &*left.0.borrow(),
                    &*right.0.borrow(),
                ),
                // If string, check if they are equal based on string values
                (LuaVal::LuaString(s1), LuaVal::LuaString(s2)) => {
                    Ok(LuaValue::new(LuaVal::LuaBool(s1 == s2)))
                }
                // If bool, check if they are equal based on bool values
                (LuaVal::LuaBool(b1), LuaVal::LuaBool(b2)) => {
                    Ok(LuaValue::new(LuaVal::LuaBool(b1 == b2)))
                }
                // If table, check if they are equal based on reference
                (LuaVal::LuaTable(_), LuaVal::LuaTable(_)) => {
                    // TODO: check after table is implemented
                    Ok(LuaValue::new(LuaVal::LuaBool(Rc::ptr_eq(
                        &left.0, &right.0,
                    ))))
                }
                // If function, check if they are equal based on reference
                (LuaVal::Function(_), LuaVal::Function(_)) => Ok(LuaValue::new(LuaVal::LuaBool(
                    Rc::ptr_eq(&left.0, &right.0),
                ))),
                _ => Ok(LuaValue::new(LuaVal::LuaBool(false))),
            }
        }

        fn less_or_greater_than<'a>(
            left: LuaValue<'a>,
            right: LuaValue<'a>,
            is_less_than: bool,
        ) -> Result<LuaValue<'a>, ASTExecError> {
            match (&*left.0.borrow(), &*right.0.borrow()) {
                // If number, check if they are equal based on mathematical values
                (LuaVal::LuaNum(_, _), LuaVal::LuaNum(_, _)) => execute_arithmetic(
                    |i1, i2| {
                        if is_less_than {
                            IntFloatBool::Bool(i1 < i2)
                        } else {
                            IntFloatBool::Bool(i1 > i2)
                        }
                    },
                    |f1, f2| {
                        if is_less_than {
                            IntFloatBool::Bool(f1 < f2)
                        } else {
                            IntFloatBool::Bool(f1 > f2)
                        }
                    },
                    &*left.0.borrow(),
                    &*right.0.borrow(),
                ),
                // If string, check if they are equal based on string values
                (LuaVal::LuaString(s1), LuaVal::LuaString(s2)) => Ok({
                    if is_less_than {
                        LuaValue::new(LuaVal::LuaBool(s1 < s2))
                    } else {
                        LuaValue::new(LuaVal::LuaBool(s1 > s2))
                    }
                }),
                _ => Err(ASTExecError(
                    "Cannot compare two values due to types".to_string(),
                )),
            }
        }

        let left = LuaValue::extract_first_return_val(left.eval(env)?);
        match op {
            BinOp::Add => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                let exec_ints = |i1: i64, i2: i64| IntFloatBool::Int(i1 + i2);
                let exec_floats = |f1: f64, f2: f64| IntFloatBool::Float(f1 + f2);
                let result = execute_arithmetic(exec_ints, exec_floats, &*left.0.borrow(), &*right.0.borrow());
                result
            }
            BinOp::Sub => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                let exec_ints = |i1: i64, i2: i64| IntFloatBool::Int(i1 - i2);
                let exec_floats = |f1: f64, f2: f64| IntFloatBool::Float(f1 - f2);
                let result = execute_arithmetic(exec_ints, exec_floats, &*left.0.borrow(), &*right.0.borrow());
                result
            }
            BinOp::Mult => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                let exec_ints = |i1: i64, i2: i64| IntFloatBool::Int(i1 * i2);
                let exec_floats = |f1: f64, f2: f64| IntFloatBool::Float(f1 * f2);
                let result = execute_arithmetic(exec_ints, exec_floats, &*left.0.borrow(), &*right.0.borrow());
                result
            }
            BinOp::Div => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                let exec_ints = |i1: i64, i2: i64| IntFloatBool::Float(i1 as f64 / i2 as f64);
                let exec_floats = |f1: f64, f2: f64| IntFloatBool::Float(f1 / f2);
                let result = execute_arithmetic(exec_ints, exec_floats, &*left.0.borrow(), &*right.0.borrow());
                result
            }
            BinOp::IntegerDiv => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                let exec_ints = |i1: i64, i2: i64| IntFloatBool::Int(i1 / i2);
                let exec_floats = |f1: f64, f2: f64| IntFloatBool::Int((f1 / f2).floor() as i64);
                let result = execute_arithmetic(exec_ints, exec_floats, &*left.0.borrow(), &*right.0.borrow());
                result
            }
            BinOp::Pow => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                let exec_ints = |i1: i64, i2: i64| {
                    let i1 = i1 as f64;
                    let i2 = i2 as f64;
                    IntFloatBool::Float(i1.powf(i2))
                };
                let exec_floats = |f1: f64, f2: f64| IntFloatBool::Float(f1.powf(f2));
                let result = execute_arithmetic(exec_ints, exec_floats, &*left.0.borrow(), &*right.0.borrow());
                result
            }
            BinOp::Mod => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                let exec_ints = |i1: i64, i2: i64| IntFloatBool::Int(i1 % i2);
                let exec_floats = |f1: f64, f2: f64| IntFloatBool::Float(f1 % f2);
                let result = execute_arithmetic(exec_ints, exec_floats, &*left.0.borrow(), &*right.0.borrow());
                result
            }
            BinOp::BitAnd => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                Ok(LuaValue::new(LuaVal::LuaNum(
                    (left.into_int()? & right.into_int()?).to_be_bytes(),
                    false,
                )))
            }
            BinOp::BitXor => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                Ok(LuaValue::new(LuaVal::LuaNum(
                    (left.into_int()? ^ right.into_int()?).to_be_bytes(),
                    false,
                )))
            }
            BinOp::BitOr => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                Ok(LuaValue::new(LuaVal::LuaNum(
                    (left.into_int()? | right.into_int()?).to_be_bytes(),
                    false,
                )))
            }
            BinOp::ShiftRight => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                Ok(LuaValue::new(LuaVal::LuaNum(
                    (left.into_int()? >> right.into_int()?).to_be_bytes(),
                    false,
                )))
            }
            BinOp::ShiftLeft => {
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                Ok(LuaValue::new(LuaVal::LuaNum(
                    (left.into_int()? << right.into_int()?).to_be_bytes(),
                    false,
                )))
            }
            BinOp::Concat => {
                // If both operands are strings or numbers, then the numbers are converted to strings in a non-specified format.
                // Otherwise, the __concat metamethod is called (in our case, return error).
                let right = LuaValue::extract_first_return_val(right.eval(env)?);
                Ok(LuaValue::new(LuaVal::LuaString(format!(
                    "{}{}",
                    left.into_string()?,
                    right.into_string()?
                ))))
            }
            BinOp::LessThan => less_or_greater_than(
                left,
                LuaValue::extract_first_return_val(right.eval(env)?),
                true,
            ),
            BinOp::LessEq => less_or_greater_than(
                left,
                LuaValue::extract_first_return_val(right.eval(env)?),
                false,
            )?
            .negate_bool(),
            BinOp::GreaterThan => less_or_greater_than(
                left,
                LuaValue::extract_first_return_val(right.eval(env)?),
                false,
            ),
            BinOp::GreaterEq => less_or_greater_than(
                left,
                LuaValue::extract_first_return_val(right.eval(env)?),
                true,
            )?
            .negate_bool(),
            BinOp::Equal => equal(left, LuaValue::extract_first_return_val(right.eval(env)?)),
            BinOp::NotEqual => {
                equal(left, LuaValue::extract_first_return_val(right.eval(env)?))?.negate_bool()
            }
            BinOp::LogicalAnd => {
                if left.is_false() {
                    Ok(left)
                } else {
                    // If left is true, return value on the right
                    Ok(LuaValue::extract_first_return_val(right.eval(env)?))
                }
            }
            BinOp::LogicalOr => {
                if left.is_true() {
                    Ok(left)
                } else {
                    // If left is true, return value on the right
                    Ok(LuaValue::extract_first_return_val(right.eval(env)?))
                }
            }
        }
    }
}

impl PrefixExp {
    pub fn eval<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<Vec<LuaValue<'a>>, ASTExecError> {
        match self {
            PrefixExp::Var(var) => {
                match var {
                    Var::NameVar(name) => match env.get(&name) {
                        Some(val) => Ok(vec![val.clone()]),
                        None => Ok(vec![LuaValue::new(LuaVal::LuaNil)]),
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
                Ok(return_vals)
            }
            PrefixExp::Exp(exp) => Ok(exp.eval(env)?),
        }
    }

    pub fn capture_variables<'a>(&self, env: &Env<'a>) -> Vec<(String, LuaValue<'a>)> {
        match self {
            PrefixExp::Var(var) => var.capture_variables(env),
            PrefixExp::FunctionCall(funcall) => funcall.capture_variables(env),
            PrefixExp::Exp(exp) => exp.capture_variables(env),
        }
    }
}

impl Var {
    pub fn capture_variables<'a>(&self, env: &Env<'a>) -> Vec<(String, LuaValue<'a>)> {
        match self {
            Var::NameVar(name) => match env.get(&name) {
                // Value is cloned as Rc, which is not huge overhead
                Some(val) => {
                    vec![(name.clone(), val.clone())]
                }
                // If value is not found, not captured
                None => vec![],
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
}

impl FunctionCall {
    pub fn exec<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<Vec<LuaValue<'a>>, ASTExecError> {
        match self {
            FunctionCall::Standard((func, args)) => {
                match &*LuaValue::extract_first_return_val((*func).eval(env)?).0.borrow() {
                    LuaVal::Function(LuaFunction {
                        par_list,
                        block,
                        captured_variables,
                    }) => {
                        // Evaluate arguments first
                        let args = args.eval(env)?;

                        // Create environment for function
                        let mut func_env =
                            Env::vec_to_env(&captured_variables, env.get_global_env().clone());

                        // Extend function environment with function arguments
                        func_env.extend_local_env();
                        let par_length = par_list.0.len();
                        let arg_length = args.len();
                        for i in 0..par_length {
                            // Arguments are locally scoped
                            if i >= arg_length {
                                func_env.insert_local(
                                    par_list.0[i].clone(),
                                    LuaValue::new(LuaVal::LuaNil),
                                );
                            } else {
                                func_env.insert_local(par_list.0[i].clone(), args[i].clone());
                            }
                        }

                        let result = block.exec(&mut func_env)?;

                        // Remove arguments from the environment
                        func_env.pop_local_env();
                        match result {
                            Some(vals) => Ok(vals),
                            None => Err(ASTExecError(format!(
                                "Break statement can be only used in while, repeat, or for loop"
                            ))),
                        }
                    }
                    LuaVal::Print => {
                        let args = args.eval(env)?;
                        let mut stdout = io::stdout().lock();
                        FunctionCall::print_fn(args, &mut stdout)
                    }
                    LuaVal::Read => {
                        // TODO: use io::stdin().read_line(&mut input)
                        unimplemented!()
                    }
                    _ => {
                        return Err(ASTExecError(format!(
                            "Cannot call non-function value with arguments."
                        )))
                    }
                }
            }
            FunctionCall::Method((object, method_name, args)) => {
                // TODO: implement after table
                // TODO: Lua object is basically a table
                unimplemented!()
            }
        }
    }

    fn print_fn<'a, W>(
        args: Vec<LuaValue>,
        stdout: &mut W,
    ) -> Result<Vec<LuaValue<'a>>, ASTExecError>
    where
        W: std::io::Write,
    {
        let mut i = 0;
        for arg in args.iter() {
            match if i == args.len() - 1 {
                writeln!(stdout, "{}", arg)
            } else {
                write!(stdout, "{} ", arg)
            } {
                Ok(_) => {}
                Err(_) => {
                    return Err(ASTExecError(format!(
                        "Cannot print value of type {:?}",
                        arg.0
                    )))
                }
            }
            i += 1;
        }
        Ok(vec![])
    }

    pub fn capture_variables<'a>(&self, env: &Env<'a>) -> Vec<(String, LuaValue<'a>)> {
        match self {
            FunctionCall::Standard((func, args)) => {
                let mut captured_vars = func.capture_variables(env);
                match args {
                    Args::ExpList(exps_list) => {
                        for exp in exps_list.iter() {
                            captured_vars.append(&mut exp.capture_variables(env));
                        }
                    }
                    // TODO: implement after table
                    Args::TableConstructor(table) => unimplemented!(),
                    Args::LiteralString(_) => {
                        // Do nothing
                    }
                }
                captured_vars
            }
            FunctionCall::Method((object, method_name, args)) => {
                // TODO: implement after table
                unimplemented!()
            }
        }
    }
}

impl Args {
    fn eval<'a>(&'a self, env: &mut Env<'a>) -> Result<Vec<LuaValue<'a>>, ASTExecError> {
        match self {
            Args::ExpList(exps_list) => {
                let mut args = Vec::with_capacity(exps_list.len());
                let mut i = 0;
                for exp in exps_list.iter() {
                    // For each argument, only the first return value is used,
                    // but last argument can use multiple return values
                    if i == exps_list.len() - 1 {
                        args.append(&mut exp.eval(env)?);
                    } else {
                        args.push(LuaValue::extract_first_return_val(exp.eval(env)?));
                    }
                    i += 1;
                }
                Ok(args)
            }
            Args::TableConstructor(table) => {
                // TODO: implement after table (single argument of table)
                unimplemented!()
            }
            Args::LiteralString(s) => Ok(vec![LuaValue::new(LuaVal::LuaString(s.clone()))]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    // Helper functions
    fn var_exp(name: &str) -> Expression {
        Expression::PrefixExp(Box::new(PrefixExp::Var(Var::NameVar(name.to_string()))))
    }
    fn lua_integer<'a>(n: i64) -> Vec<LuaValue<'a>> {
        vec![LuaValue::new(LuaVal::LuaNum(n.to_be_bytes(), false))]
    }
    fn lua_integers<'a>(nums: Vec<i64>) -> Vec<LuaValue<'a>> {
        let mut v = Vec::with_capacity(nums.len());
        for n in nums {
            v.push(LuaValue::new(LuaVal::LuaNum(n.to_be_bytes(), false)));
        }
        v
    }
    fn lua_float<'a>(n: f64) -> Vec<LuaValue<'a>> {
        vec![LuaValue::new(LuaVal::LuaNum(n.to_be_bytes(), true))]
    }
    fn lua_nil<'a>() -> Vec<LuaValue<'a>> {
        vec![LuaValue::new(LuaVal::LuaNil)]
    }
    fn lua_false<'a>() -> Vec<LuaValue<'a>> {
        vec![LuaValue::new(LuaVal::LuaBool(false))]
    }
    fn lua_true<'a>() -> Vec<LuaValue<'a>> {
        vec![LuaValue::new(LuaVal::LuaBool(true))]
    }
    fn lua_string<'a>(s: &str) -> Vec<LuaValue<'a>> {
        vec![LuaValue::new(LuaVal::LuaString(s.to_string()))]
    }
    fn lua_function<'a>(
        par_list: &'a ParList,
        block: &'a Block,
        env: &Env<'a>,
    ) -> Vec<LuaValue<'a>> {
        let captured_variables = block.capture_variables(env);
        vec![LuaValue::new(LuaVal::Function(LuaFunction {
            par_list,
            block,
            captured_variables,
        }))]
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
    fn test_eval_func_def() {
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
            Ok(lua_function(&par_list, &block, &env))
        );
    }

    #[test]
    fn test_eval_func_call() {
        let mut env = Env::new();

        // Set statements
        let varlist = vec![Var::NameVar("a".to_string()), Var::NameVar("b".to_string())];
        let explist = vec![
            Expression::Numeral(Numeral::Integer(30)),
            Expression::Numeral(Numeral::Integer(20)),
        ];
        let stat = Statement::Assignment((varlist, explist, false));
        let return_stat = Some(vec![var_exp("test"), var_exp("a"), var_exp("b")]);

        let par_list = ParList(vec![String::from("test")], false);
        let block = Block {
            statements: vec![stat],
            return_stat: return_stat,
        };

        env.insert_global(
            String::from("f"),
            LuaValue::extract_first_return_val(lua_function(&par_list, &block, &env)),
        );
        let args = Args::ExpList(vec![Expression::Numeral(Numeral::Integer(100))]);
        let func_call = FunctionCall::Standard((
            Box::new(PrefixExp::Var(Var::NameVar("f".to_string()))),
            args,
        ));
        let exp = PrefixExp::FunctionCall(func_call.clone());

        // f(100) executes a = 30, b = 20, return test
        assert_eq!(exp.eval(&mut env), Ok(lua_integers(vec![100, 30, 20])));

        // Function with return values of function call
        let func_call_exp = Expression::PrefixExp(Box::new(PrefixExp::FunctionCall(func_call)));
        let par_list = ParList(vec![], false);
        let block = Block {
            statements: vec![],
            return_stat: Some(vec![
                func_call_exp.clone(),
                func_call_exp.clone(),
                func_call_exp.clone(),
            ]),
        };
        env.insert_global(
            String::from("f2"),
            LuaValue::extract_first_return_val(lua_function(&par_list, &block, &env)),
        );
        let func_call2 = PrefixExp::FunctionCall(FunctionCall::Standard((
            Box::new(PrefixExp::Var(Var::NameVar("f2".to_string()))),
            Args::ExpList(vec![]),
        )));
        // Each return value return one of the values, but last one return all
        assert_eq!(
            func_call2.eval(&mut env),
            Ok(lua_integers(vec![100, 100, 100, 30, 20]))
        );

        // Function with taking function that return multiple values as arguments
        let par_list = ParList(
            vec![String::from("a"), String::from("b"), String::from("c")],
            false,
        );
        let block = Block {
            statements: vec![],
            return_stat: Some(vec![var_exp("a"), var_exp("b"), var_exp("c")]),
        };
        env.insert_global(
            String::from("f3"),
            LuaValue::extract_first_return_val(lua_function(&par_list, &block, &env)),
        );
        let args = Args::ExpList(vec![func_call_exp.clone(), func_call_exp.clone()]);
        let func_call3 = PrefixExp::FunctionCall(FunctionCall::Standard((
            Box::new(PrefixExp::Var(Var::NameVar("f3".to_string()))),
            args,
        )));
        // Each argument take one return value of each expression except last one
        assert_eq!(
            func_call3.eval(&mut env),
            Ok(lua_integers(vec![100, 100, 30]))
        );
    }

    #[test]
    fn test_eval_print() {
        let mut env = Env::new();

        // Integer, float, boolean, string, nil, function
        // TODO: add table after implementing table (print reference)
        let par_list = ParList(vec![], false);
        let block = Block {
            statements: vec![],
            return_stat: None,
        };
        let f = LuaValue::extract_first_return_val(lua_function(&par_list, &block, &env));
        env.insert_global("f".to_string(), f);
        let args = Args::ExpList(vec![
            Expression::Numeral(Numeral::Integer(10)),
            Expression::Numeral(Numeral::Float(10.1)),
            Expression::False,
            Expression::LiteralString("Hello World!".to_string()),
            Expression::Nil,
            var_exp("f"),
        ]);
        let print_call = FunctionCall::Standard((
            Box::new(PrefixExp::Var(Var::NameVar("print".to_string()))),
            args.clone(),
        ));
        let print_exp = PrefixExp::FunctionCall(print_call);

        // Capture the output of `print`
        assert_eq!(print_exp.eval(&mut env), Ok(vec![]));

        let mut output = Vec::new();
        assert_eq!(
            FunctionCall::print_fn(args.eval(&mut env).unwrap(), &mut output),
            Ok(vec![])
        );
        let func_reference = &*env.get_global("f").unwrap().0.borrow();
        let func_reference = if let LuaVal::Function(f) = func_reference
        {
            f
        } else {
            unreachable!("Expected function")
        };
        assert_eq!(
            String::from_utf8(output).unwrap(),
            format!("10 10.1 false Hello World! nil {:p}\n", func_reference)
        );
    }

    #[test]
    fn test_eval_bin_add() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Add, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(30)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Add, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(30.1)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(10.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Add, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(30.1)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Float(0.9));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Add, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(11 as f64)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::LiteralString("Can't add string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Add, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot execute opration on values that are not numbers".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_sub() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Sub, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(-10)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Sub, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(-9.9)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(10.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Sub, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(9.9)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Float(0.9));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Sub, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(9.2)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::LiteralString("Can't subtract with string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Sub, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot execute opration on values that are not numbers".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_mult() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Mult, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(200)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Mult, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(202.0)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(-10.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Mult, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(-202.0)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Float(0.9));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Mult, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(9.09)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::LiteralString("Can't multipy string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Sub, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot execute opration on values that are not numbers".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_div() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Div, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(2.0)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Div, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(1.01)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(10.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Div, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(20 as f64 / 10.1)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Float(0.9));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Div, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(10.1 / 0.9)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::LiteralString("Can't float divide with string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Div, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot execute opration on values that are not numbers".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_int_div() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::IntegerDiv, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(2)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::IntegerDiv, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(1)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(10.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::IntegerDiv, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(1)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Float(0.9));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::IntegerDiv, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(11)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::LiteralString("Can't floor divide with string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::IntegerDiv, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot execute opration on values that are not numbers".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_pow() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(2));
        let right = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Pow, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(1024.0)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Integer(3));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Pow, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(1030.301)));

        let left = Expression::Numeral(Numeral::Integer(2));
        let right = Expression::Numeral(Numeral::Float(10.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Pow, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(2.0_f64.powf(10.1))));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Float(0.9));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Pow, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(10.1_f64.powf(0.9))));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::LiteralString("Can't power with string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Pow, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot execute opration on values that are not numbers".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_mod() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Mod, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(0)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Mod, Box::new(right)));
        // In Rust, 10.1 % 10.0 = 0.09999999999999964
        assert_eq!(exp.eval(&mut env), Ok(lua_float(10.1 % 10.0)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(10.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Mod, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(9.9)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::Numeral(Numeral::Float(0.9));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Mod, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(10.1 % 0.9)));

        let left = Expression::Numeral(Numeral::Float(10.1));
        let right = Expression::LiteralString("Can't mod with string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Mod, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot execute opration on values that are not numbers".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_bitand() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(13));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitAnd, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(4)));

        let left = Expression::Numeral(Numeral::Float(20.0));
        let right = Expression::Numeral(Numeral::Float(13.0));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitAnd, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(4)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(13.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitAnd, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert float that does not have exact integer value to integer"
                    .to_string()
            ))
        );

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::LiteralString("Can't bitwise and with string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitAnd, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert value to integer (types cannot be converted)".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_bitxor() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(13));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitXor, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(25)));

        let left = Expression::Numeral(Numeral::Float(20.0));
        let right = Expression::Numeral(Numeral::Float(13.0));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitXor, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(25)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(13.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitXor, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert float that does not have exact integer value to integer"
                    .to_string()
            ))
        );

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::LiteralString("Can't bitwise and with string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitXor, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert value to integer (types cannot be converted)".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_bitor() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(13));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitOr, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(29)));

        let left = Expression::Numeral(Numeral::Float(20.0));
        let right = Expression::Numeral(Numeral::Float(13.0));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitOr, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(29)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(13.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitOr, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert float that does not have exact integer value to integer"
                    .to_string()
            ))
        );

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::LiteralString("Can't bitwise and with string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::BitOr, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert value to integer (types cannot be converted)".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_bitsl() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(13));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::ShiftLeft, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(163840)));

        let left = Expression::Numeral(Numeral::Float(20.0));
        let right = Expression::Numeral(Numeral::Float(13.0));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::ShiftLeft, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(163840)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(13.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::ShiftLeft, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert float that does not have exact integer value to integer"
                    .to_string()
            ))
        );

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::LiteralString("Can't bitwise and with string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::ShiftLeft, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert value to integer (types cannot be converted)".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_bitsr() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::ShiftRight, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(5)));

        let left = Expression::Numeral(Numeral::Float(20.0));
        let right = Expression::Numeral(Numeral::Float(2.0));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::ShiftRight, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(5)));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Float(2.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::ShiftRight, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert float that does not have exact integer value to integer"
                    .to_string()
            ))
        );

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::LiteralString("Can't bitwise and with string".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::ShiftRight, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert value to integer (types cannot be converted)".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_concat() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Concat, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_string("202")));

        let left = Expression::Numeral(Numeral::Float(20.0));
        let right = Expression::Numeral(Numeral::Float(2.0));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Concat, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_string("20.02.0")));

        let left = Expression::Numeral(Numeral::Float(20.0));
        let right = Expression::LiteralString("test".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Concat, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_string("20.0test")));

        let left = Expression::LiteralString("Hello ".to_string());
        let right = Expression::LiteralString("World!".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Concat, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_string("Hello World!")));

        let left = Expression::Nil;
        let right = Expression::Numeral(Numeral::Float(2.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Concat, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert value to String (types cannot be converted)".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_equal() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Equal, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Equal, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::Numeral(Numeral::Float(2.0));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Equal, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::Nil;
        let right = Expression::Nil;
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Equal, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::LiteralString("Same content".to_string());
        let right = Expression::LiteralString("Same content".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Equal, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        // Function with same content but not same reference
        let left = Expression::FunctionDef((
            ParList(vec![], false),
            Block {
                statements: vec![],
                return_stat: None,
            },
        ));
        let right = Expression::FunctionDef((
            ParList(vec![], false),
            Block {
                statements: vec![],
                return_stat: None,
            },
        ));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Equal, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        // Function with same reference
        let stat = Statement::FunctionDecl((
            "f".to_string(),
            ParList(vec![], false),
            Block {
                statements: vec![],
                return_stat: None,
            },
        ));
        stat.exec(&mut env).unwrap();
        let exp =
            Expression::BinaryOp((Box::new(var_exp("f")), BinOp::Equal, Box::new(var_exp("f"))));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        // TODO: add test for table after implementing table

        let left = Expression::LiteralString("Different types".to_string());
        let right = Expression::Numeral(Numeral::Float(2.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::Equal, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));
    }

    #[test]
    fn test_eval_bin_not_equal() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::NotEqual, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::NotEqual, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::Numeral(Numeral::Float(2.0));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::NotEqual, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::Nil;
        let right = Expression::Nil;
        let exp = Expression::BinaryOp((Box::new(left), BinOp::NotEqual, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::LiteralString("Same content".to_string());
        let right = Expression::LiteralString("Same content".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::NotEqual, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        // Function with same content but not same reference
        let left = Expression::FunctionDef((
            ParList(vec![], false),
            Block {
                statements: vec![],
                return_stat: None,
            },
        ));
        let right = Expression::FunctionDef((
            ParList(vec![], false),
            Block {
                statements: vec![],
                return_stat: None,
            },
        ));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::NotEqual, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        // Function with same reference
        let stat = Statement::FunctionDecl((
            "f".to_string(),
            ParList(vec![], false),
            Block {
                statements: vec![],
                return_stat: None,
            },
        ));
        stat.exec(&mut env).unwrap();
        let exp = Expression::BinaryOp((
            Box::new(var_exp("f")),
            BinOp::NotEqual,
            Box::new(var_exp("f")),
        ));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        // TODO: add test for table after implementing table

        let left = Expression::LiteralString("Different types".to_string());
        let right = Expression::Numeral(Numeral::Float(2.1));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::NotEqual, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));
    }

    #[test]
    fn test_eval_bin_less_than() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LessThan, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::Numeral(Numeral::Float(2.0));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LessThan, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::Numeral(Numeral::Float(2.0));
        let right = Expression::Numeral(Numeral::Integer(4));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LessThan, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::LiteralString("abc".to_string());
        let right = Expression::LiteralString("cba".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LessThan, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::LiteralString("abc".to_string());
        let right = Expression::Nil;
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LessThan, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot compare two values due to types".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_less_equal() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LessEq, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::Numeral(Numeral::Float(2.0));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LessEq, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::Numeral(Numeral::Float(2.0));
        let right = Expression::Numeral(Numeral::Integer(4));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LessEq, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::LiteralString("abc".to_string());
        let right = Expression::LiteralString("cba".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LessEq, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::LiteralString("abc".to_string());
        let right = Expression::Nil;
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LessEq, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot compare two values due to types".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_greater_than() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::GreaterThan, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::Numeral(Numeral::Float(2.0));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::GreaterThan, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::Numeral(Numeral::Float(2.0));
        let right = Expression::Numeral(Numeral::Integer(4));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::GreaterThan, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::LiteralString("abc".to_string());
        let right = Expression::LiteralString("cba".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::GreaterThan, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::LiteralString("abc".to_string());
        let right = Expression::Nil;
        let exp = Expression::BinaryOp((Box::new(left), BinOp::GreaterThan, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot compare two values due to types".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_greater_equal() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(20));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::GreaterEq, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::Numeral(Numeral::Float(2.0));
        let right = Expression::Numeral(Numeral::Integer(2));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::GreaterEq, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let left = Expression::Numeral(Numeral::Float(2.0));
        let right = Expression::Numeral(Numeral::Integer(4));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::GreaterEq, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::LiteralString("abc".to_string());
        let right = Expression::LiteralString("cba".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::GreaterEq, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::LiteralString("abc".to_string());
        let right = Expression::Nil;
        let exp = Expression::BinaryOp((Box::new(left), BinOp::GreaterEq, Box::new(right)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot compare two values due to types".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_bin_logical_and() {
        let mut env = Env::new();

        let left = Expression::Nil;
        let right = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LogicalAnd, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_nil()));

        let left = Expression::False;
        // right should return error when evaluated
        let right = Expression::BinaryOp((
            Box::new(Expression::LiteralString("abc".to_string())),
            BinOp::GreaterEq,
            Box::new(Expression::Nil),
        ));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LogicalAnd, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::False;
        let right = Expression::Nil;
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LogicalAnd, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LogicalAnd, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(20)));
    }

    #[test]
    fn test_eval_bin_logical_or() {
        let mut env = Env::new();

        let left = Expression::Numeral(Numeral::Integer(10));
        let right = Expression::Numeral(Numeral::Integer(20));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LogicalOr, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(10)));

        let left = Expression::Numeral(Numeral::Integer(10));
        // right should return error when evaluated
        let right = Expression::BinaryOp((
            Box::new(Expression::LiteralString("abc".to_string())),
            BinOp::GreaterEq,
            Box::new(Expression::Nil),
        ));
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LogicalOr, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(10)));

        let left = Expression::Nil;
        let right = Expression::LiteralString("a".to_string());
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LogicalOr, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_string("a")));

        let left = Expression::False;
        let right = Expression::Nil;
        let exp = Expression::BinaryOp((Box::new(left), BinOp::LogicalOr, Box::new(right)));
        assert_eq!(exp.eval(&mut env), Ok(lua_nil()));
    }

    #[test]
    fn test_eval_un_negate() {
        let mut env = Env::new();

        let exp = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::UnaryOp((UnOp::Negate, Box::new(exp)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(-10)));

        let exp = Expression::Numeral(Numeral::Float(10.1));
        let exp = Expression::UnaryOp((UnOp::Negate, Box::new(exp)));
        assert_eq!(exp.eval(&mut env), Ok(lua_float(-10.1)));

        let exp = Expression::LiteralString("String cannot be negated".to_string());
        let exp = Expression::UnaryOp((UnOp::Negate, Box::new(exp)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot negate values that are not numbers".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_un_not() {
        let mut env = Env::new();

        let exp = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::UnaryOp((UnOp::LogicalNot, Box::new(exp)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let exp =
            Expression::LiteralString("Everything other than nil and false is true".to_string());
        let exp = Expression::UnaryOp((UnOp::LogicalNot, Box::new(exp)));
        assert_eq!(exp.eval(&mut env), Ok(lua_false()));

        let exp = Expression::False;
        let exp = Expression::UnaryOp((UnOp::LogicalNot, Box::new(exp)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));

        let exp = Expression::Nil;
        let exp = Expression::UnaryOp((UnOp::LogicalNot, Box::new(exp)));
        assert_eq!(exp.eval(&mut env), Ok(lua_true()));
    }

    #[test]
    fn test_eval_un_length() {
        let mut env = Env::new();

        let exp = Expression::LiteralString("Let's get string length".to_string());
        let exp = Expression::UnaryOp((UnOp::Length, Box::new(exp)));
        assert_eq!(
            exp.eval(&mut env),
            Ok(lua_integer("Let's get string length".len() as i64))
        );

        // TODO: impelement after table (add test case for table)

        let exp = Expression::Numeral(Numeral::Integer(10));
        let exp = Expression::UnaryOp((UnOp::Length, Box::new(exp)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot get length of value that is not a string or table".to_string()
            ))
        );
    }

    #[test]
    fn test_eval_un_bitnot() {
        let mut env = Env::new();

        let exp = Expression::Numeral(Numeral::Integer(100));
        let exp = Expression::UnaryOp((UnOp::BitNot, Box::new(exp)));
        assert_eq!(exp.eval(&mut env), Ok(lua_integer(-101)));

        let exp = Expression::LiteralString("Let's bitwise not string".to_string());
        let exp = Expression::UnaryOp((UnOp::BitNot, Box::new(exp)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert value to integer (types cannot be converted)".to_string()
            ))
        );

        let exp = Expression::Numeral(Numeral::Float(10.04));
        let exp = Expression::UnaryOp((UnOp::BitNot, Box::new(exp)));
        assert_eq!(
            exp.eval(&mut env),
            Err(ASTExecError(
                "Cannot convert float that does not have exact integer value to integer"
                    .to_string()
            ))
        );
    }

    #[test]
    fn test_capture_variables() {
        let mut env = Env::new();
        env.insert_local(
            "a".to_string(),
            LuaValue::extract_first_return_val(lua_integer(10)),
        );
        env.insert_local(
            "b".to_string(),
            LuaValue::extract_first_return_val(lua_integer(20)),
        );
        env.insert_local(
            "c".to_string(),
            LuaValue::extract_first_return_val(lua_integer(30)),
        );

        let block = Block {
            statements: vec![],
            return_stat: Some(vec![var_exp("a"), var_exp("b"), var_exp("c"), var_exp("d")]),
        };

        let captured_varaibles = block.capture_variables(&env);
        env.pop_local_env();
        let func_env = Env::vec_to_env(&captured_varaibles, env.get_global_env().clone());
        assert_eq!(
            func_env.get("a"),
            Some(&LuaValue::extract_first_return_val(lua_integer(10)))
        );
        assert_eq!(
            func_env.get("b"),
            Some(&LuaValue::extract_first_return_val(lua_integer(20)))
        );
        assert_eq!(
            func_env.get("c"),
            Some(&LuaValue::extract_first_return_val(lua_integer(30)))
        );
        assert_eq!(func_env.get("d"), None);
    }
}
