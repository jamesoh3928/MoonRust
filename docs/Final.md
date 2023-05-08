# MoonRust

![alt text](../assets/moon.jpg)

MoonRust is a Lua interpreter implemented in Rust. Lua means "moon" in Portuguese.

Team members:

- James Oh
- Matthew DellaNeve
- Renee Veit

## Summary Description

The goal of this project was to build an interpreter that will execute a subset of Lua given a file to read. The details of the functionality are specified in the _Project Breakdown_ section. (Note: our implementation logic differs from Lua's reference implementation since we are not supporting the full features of Lua. Our interpreter should be able to run a simple Lua standalone program).

### Demo

[MoonRust Demo](https://youtu.be/mAzZ4ySGXfQ)

## Project Breakdown

### Implementation

We were able to finish our MVP by implementing the following features:

1. Parser (excluding syntactic sugar such as certain literal string forms and some forms of table constructor)
2. Variable assignments
3. Binary expression evaluation
4. Unary expression evaluation
5. Control statement evaluation (if, else, break)
6. Loop statement evaluation (for, while, repeat - excluding for generics)
7. Function definition/call evaluation
8. Visibility rules and scoping
9. Table evaluation
10. Some standard library functions (print, read, random)

### Challenges/Lessons

We had multiple challenges throughout the project and we found solutions for most of them by trying different approaches. The details of the different approaches we took can be found in the _Different Approaches_ section.

1. **Parsing left recursion**

Problem: Nom is a top-down parser, and is thus unable to recognize grammars with left recursion. The specification for the syntax in the Lua manual (https://www.lua.org/manual/5.4/manual.html#9) contains a few ambiguous and left-recursive rules, meaning that our initial attempt to parse directly as it's written in the manual causes the parser to overflow the stack.

Lesson: For expressions, the main way we factored out left recursion is by establishing a sort of parsing hierarchy according to Lua's operator precedence rules. In effect, this forces expression "atoms" like primitive values, table constructors, etc. to get parsed first, and certain types of expressions (arithmetic, and expressions, or expressions, etc.) get parsed in order of the operator precedence. This eliminates any ambiguity. Prefix expressions also had left recursion issues because the `prefixexp` rule was mutually recursive with the `var` and `functioncall` rules. We solved this issue by flattening out those rules into one unambiguous rule. An unfortunate complication of this was that we couldn't update our abstract syntax tree to accomodate this (since doing so would have broken parts of the interpreter code). We worked around this by parsing prefix expressions into an intermediate data structure that we then convert back into our AST representation. The overall lesson we learned was that parsing is not as straightforward as we initially thought. Language grammars can be ambiguous, so it's important that we really understand their structure before we try parsing with them.

2. **Implementing Environment**

Problem: Unlike Rust, multiple Lua variables can own the same value. We had to use a lot of `Rc`s and `RefCell`s to allow multiple owners and while still obeying the behavior of copy types and reference types. Also, a Lua function can act as a closure, meaning it can capture variables when it is defined. This means that we either have to identify variables that are being captured, or we capture the environment where the closure is defined. This was a tricky problem to solve, but we found a solution by wrapping `EnvTable` with `Rc` and `RefCell` so that the function can capture an environment where it is defined and separating scopes in `Env` with `None` to avoid capturing variables that are defined after the closure. More details of our solution can be found in the _Different Approaches_ section.

Lesson: Although Rust has strict ownership and alias rules, we can use `Rc` and `RefCell` to enforce some behaviors that require multiple owners. However, this may increase some overhead since the ownership rules will be enforced at runtime. Another lesson we learned was that there are various ways to establish invariants for `Rc` and `RefCell`, and we need to be careful which invariant we establish (eg. some invariant may require to use of `unsafe` while others do not).

3. **Lifetime parameters**

Problem: Since a function can be called multiple times, we need to store a reference to the function body inside the AST when the function is defined (same for loops). This means a Lua function needs to point to a `Block` inside the AST. This also means that we need **lifetime parameters** because our types will be storing references. A Lua function is pointing to a block in the AST. A Lua function is also a variant of the `LuaValue` enum, and our environment stores `LuaValue`s, which means that we need to expand lifetime parameters across all of these types.

Lesson: This problem enhanced our understanding of lifetime parameters, especially that they are used to link the lifetimes of different references. Additionally, linking up wrong lifetimes might give you an error message that is not directly related to the lifetime, which can be hard to debug.

4. **Some unique behaviors of Lua**

Problem: Lua has some unique behaviors. For instance, the condition of repeat-until loop in Lua can access local variables that are defined inside the loop. This required a different approach from for- or while-loops because when we are executing the body (`Block`) of the loop, we just extended the scope, and the scopes were automatically popped when the execution of the `Block` is done. Since the condition in a repeat-until loop needed to access the local variables inside that body, we could not simply pop the scope after finishing each iteration. We created another function `Block::exec_without_pop` specifically for repeat-until so that we can manually pop the scope when we are executing it. There were many behaviors like this that we needed to consider, and we updated our design constantly as we discovered the behaviors we missed.

Lesson: Having a strong understanding of Lua's semantics could have reduced our development time since we had to update our design multiple times. However, considering the limited time we had and the nature of software engineering (iterative development), we think our initial design was a good start. We learned things that we might want to consider in the initial design process for future projects. Also, setting up the testing infrastructure in the early stage can be a huge advantage since it's easy to add edge cases and check if our program is misbehaving after changes.

## Additional Details

### List of Dependencies

- nom = "7" (parsing library)
- clap = {version = "4.1", features = ["cargo", "derive"]} (command line argument parsing)
- rand = "0.8.4" (random number generator)

### Structure of the Code

The project has three main components: the entrypoint, the parser, and the interpreter.
We decided to split it up this way because it nicely separates the stages of the program's execution. Additionally, the abstract syntax tree (AST) definition is separated out into its own section. We describe these components below:

#### _Entrypoint_

The entrypoint is contained within `main.rs`. It handles command-line arguments and retrieves the program text from the given file. Once it has this text, it passes it off to the parser stage, gets the resulting AST, and then passes that to the interpreter stage for evaluation. It can also benchmark the execution time and display that result when `-s` flag is specified in command.

```
Usage: moonrust.exe [OPTIONS] <FILE.lua>

Arguments:
  <FILE.lua>  Path of the file to run

Options:
  -a, --ast    AST print flag
  -s, --stats  Report time statistics
  -h, --help   Print help
```

#### _AST_

The AST is contained entirely in the `ast.rs` file. It defines all of the data structures
that make up the AST that represents a Lua program. This AST's structure is based directly on the grammar rules in the Lua manual, where most rules directly correspond to a struct or enum defined in `ast.rs`. In hindsight, we could have simplified this to directly represent the core components of Lua as whole data structures, which might have saved us some time implementing the interpreter. However, our actual AST still worked well enough.

Each data structure in the AST also implements the `Display` and `Debug` traits. The `Display` trait was mostly useful for debugging any issues with the parser. The `Debug` trait is used to output the AST to the console after a successful parse.

<!-- - `AST`: The root of the AST. This indicates where the Lua program starts. It contains a `Block` that represents the top-level statements in the program.
- `Block`: Defines a sequence of statements in a Lua program. It may have an optional return statement, which should only be valid in the context of a function.
- `Statement`: Enumerates the different kinds of statements in Lua, which are:
  - `Assignment`: Defines variables assignment. It holds a list of variables, a list of expressions, and a boolean flag indicating if this assignment is local.
  - `FunctionCall`: Defines a function call at the statement-level. The return value is ignored, so the main interest here is any underlying code with side-effects.
  - `Break`: Represents a break statement, which is only valid within a loop.
  - `DoBlock`: Represents the definition of a new block of statements with its own scope.
  - `While`: Represents a while loop. This variant holds the test condition expression and the block within the loop
  - `Repeat`: Represents a repeat loop, which holds the block within the loop and the test condition expression.
  - `If`: Represents an if statement. The first condition expression and block are required. This variant also holds a vector of expression-block pairs representing zero or more "else if" portions. The final "else" block is optional.
  - `ForNum`: Represents a numeric for-loop. It holds a string representing the control variable, the expression that defines the starting value of the variable, and the expression for the final value of the variable. Optionally, it can have a skip expression. Finally, it holds the block representing the body of the loop.
  - `ForGeneric`: Represents a for-loop to iterate over any generic data structure. This type of data structure parses, but its evaluation panics because it's not implemented.
  - `FunctionDecl`: Represents a global function declaration, holding the name of the function, the parameters, and the body of the function.
  - `LocalFunctionDecl`: Same as `FunctionDecl` but defined locally within a given scope.
  - `Semicolon`: Represents the separation of statements (has no effect).
- `Expression`: Enumerates different kinds of Lua expressions, which are:
  - `Nil`: Represents the absence of a value
  - `True` and `False`: Represent the primitive boolean values
  - `Numeral`: Represent numeric values. Integers and floats are unified under a single data type
  - `LiteralString`: Represents a string literal, holding the contents of the string.
  - `DotDotDot`: Represents varargs within a variadic function. This parses, but its evaluation is not implemented.
  - `FunctionDef`: Represents the definition of an anonymous function, holding the function parameters and body.
  - `PrefixExp`: Represents a prefix expression (described later in this list).
  - `TableConstructor`: Represents the creation of a table, holding the list of fields into the table
  - `BinaryOp`: Represents a binary operation expression, holding the operands and the type of operation
  - `UnaryOp`: Represents a unary operation expression, holding the operand and the type of operation
- `BinOp`: Enumerates the various kinds of binary operations, including numeric, bit, comparison, logical, and string operations.
- `Unop`: Enumerates the various kinds of unary operations, including negation, logical not, bitwise not, and the length operation.
- `Numeral`: Represents a numeral. Lua numbers are represented under a unified data type, but we make a distinction between integers and floats for the purposes of handling special scenarios in the interpreter.
- `PrefixExp`: Usually represents the first part of certain kinds of expressions, but they are able to stand alone. Lua code does not have the concept of a prefix expression. This exists for the purposes of parsing. There are three kinds:
  - `Var`: Represents some form of variable access.
  - `FunctionCall`: Represents a function call.
  - `Exp`: Represents an expression wrapped in parentheses.
- `FunctionCall`: Enumerates different kinds of function calls. Each variant holds a prefix expression because the actual function/table may be stored in a variable, returned from another call, or defined in-line with the call. They also hold arguments passed to the function. The two variants are:
  - `Standard`: Represents a usual function call.
  - `Method`: Represents calling a function that's stored in a table.
- `Args`: Enumerates the different ways arguments can be passed to a function call, which are:
  - `ExpList`: Represents passing a series of expressions to a call
  - `TableConstructor`: Represents defining a table in-line and passing it directly into the call
  - `LiteralString`: Represents defining a string literal in-line and passing it directly into the call
- `ParList`: Represents the parameters to a function. This holds a list of variable names and a flag indicating if there are varargs (which are not implemented).
- `Field`: Enumerates the different forms of field declarations in tables, which are:
  - `Bracketed`: Represents `[key]=val`, which assigns an expression to either a numeric or string key.
  - `Name`: Represents `key.val`, which assigns an expression to a string key only
  - `Unnamed`: Represents an expression on its own, which is implicitly assigned to an incrementing numeric value.
- `Var`: Enumerates different ways to access values stored in variables, which are:
  - `Name`: Represents a standalone variable.
  - `Bracket`: Represents fetching a value by accessing it from a table with a numeric or string key. The table is stored behind a prefix expression.
  - `Dot`: Represents fetching a value by accessing it from a table with a string key only. The table is stored behind a prefix expression. -->

#### _Parser_

Users interface with the parser through the `parser.rs` file. This file defines the public-facing code, including the top-level `parse` function that produces the AST representing the input program. The majority of the implementation is hidden behind various sub-modules in the `parser` folder. The `expression` and `statement` modules handle parsing components related to expressions and statements respectively. Each also has a public-facing function `parse_exp` and `parse_stmt` that returns an `Expression` or `Statement` on a successful parse. The `common` module contains parsing functions that are useful for parsing sub-components of both expressions and statements. None of these functions are public-facing. The `util` module contains some utility parsing functions, which were borrowed or adapted from Nom's recipes (https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md).

The reason we decided to split up the structure of the parser this way is because expressions and statements are the dominating components of a Lua program. Most other syntactic forms are sub-components of expressions and statements, so we felt that organizing the parser accordingly made the most sense. Since expressions and statements share many of the same kinds of sub-components, we also decided that it would be natural to put those in a "common" file.

#### _Interpreter_

Just like the parser, users interface with the interpreter through the `interpreter.rs` file. Users can call the `AST::exec` method to execute the AST representation of the Lua program. The file defines `LuaValue`, `LuaVal`, and all associated functions related to the values.

The sub-modules `environment.rs`, `expression.rs`, and `statement.rs` are stored in the `interpreter` folder. The environment module defines all types and functions related to the environment (eg. `EnvTable`, `LocalEnv`, etc). The expression module contains all `eval` methods for evaluating expressions, as well as holding the corresponding unit tests. The statement module contains all `exec` methods for statements and also contains all corresponding unit tests. We separated expressions and statements into different submodules for similar reasons as the parsing module.

### Rusty code

1. **Match Expression for Enums**

Both `Expression` and `Statement` is enums with many variants In other languages, it is easy to miss all possible cases. However, Rust's match expression came in handy because the compiler enforced it to cover all possible variants. Our team noticed that this led to far fewer logic errors than developing in other programming languages.

```Rust
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
            Expression::FunctionDef((par_list, block)) => {
                let captured_env = env.get_local_env().capture_env();
                vec![LuaValue::new(LuaVal::Function(LuaFunction {
                    par_list,
                    block,
                    captured_env,
                }))]
            }
            Expression::PrefixExp(prefixexp) => prefixexp.eval(env)?,
            Expression::TableConstructor(fields) => {
                let table = build_table(fields, env)?;
                vec![LuaValue::new(LuaVal::LuaTable(table))]
            }
            Expression::BinaryOp((left, op, right)) => {
                vec![Expression::eval_binary_exp(op, left, right, env)?]
            }
            Expression::UnaryOp((op, exp)) => vec![Expression::eval_unary_exp(op, exp, env)?],
        };
        Ok(val)
    }
```

2. **Use of `Rc` and `RefCell` in the Environment**

We believe that this project is a good example of where we want to use `Rc` and `RefCell` since Lua allows multiple owners of the same value. For instance:

```Rust
pub struct EnvTable<'a>(Rc<RefCell<HashMap<String, LuaValue<'a>>>>);

pub struct LuaValue<'a>(Rc<LuaVal<'a>>);

pub struct LuaTable<'a>(RefCell<HashMap<TableKey, LuaValue<'a>>>)
```

We wrapped around `Rc<RefCell<...>>` around `HashMap` inside `EnvTable` because `EnvTable` can be owned by multiple environments (main environment or closure environments), and it can be mutated. However, we only wrapped around `Rc` inside `LuaValue` because we don't need to update the `LuaVal` inside in most cases, but multiple owners are possible. We need to update the value inside when the `LuaValue` is a table, so we added `RefCell` inside the `LuaTable`. We think this is a good use case of `Rc` and `RefCell`.

3. **`Display` trait**

We also implemented the `Display` trait for `AST` to verify if our parsing is working correctly and for `LuaValue` for debugging purposes.

```Rust
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
            LuaVal::Random => write!(f, "random"),
        }
    }
}
```

### Difficult to Express in Rust

1. Lot of `Rc` and `RefCell` were needed to implement the environment that allows multiple owners. Details can be found in the _Different Approaches_ section.
2. We wished that Rust allows default parameters for functions. For example, instead of having `Block::exec` and `Block::exec_without_pop`, we can just have the default parameter of a boolean flag which will make it easier to add new behavior to the program. However, we also understand that this might go against Rust's philosophy of zero-cost abstraction (code may be less efficient when default value is not used) and make it more difficult to debug.

### Different Approaches

1. **Parser**

When implementing the parser, we initially tried to write the parsing functions directly from the syntax specified in the Lua manual. However, this didn't work. Nom is a top-down parsing library, meaning it suffers from the same issue that all top-down parsers have: left recursion. Thus, we had several issues where running the parser would overflow the stack, since the parser would just infinitely expand some mutually recursive syntax rules. We solved this issue by changing our parsing strategy to factor out left recursion. For expressions, this was a matter of parsing according the operator precedence hierarchy. For prefix expressions, we parsed according to a "flattened" representation of the syntax in order to remove any ambiguity that was there in the original specification. One challenge there was that, since we were already using the AST to implement the interpreter, we could not change it build in this flattened prefix expression. Our workaround was to parse into an intermediate data structure, and then convert it back to our actual AST representation after a successful parse.

2. **Defining `Expression::eval` and `Statement::exec`**

For the `Expression::eval` and `Statement::exec` methods, we first defined them to take ownership of `self`, because it seemed reasonable to consume AST so that the same piece of code does not get executed multiple times. However, function bodies and code inside loops needed to be executed multiple times, which made us use `&self` instead. Borrowing immutably actually makes sense since we can ensure no one is mutating the parsed AST, but many `LuaValues` can point to the part of AST. The final signatures of the two methods look like this:

```Rust
pub fn eval<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<Vec<LuaValue<'a>>, ASTExecError>
```

```Rust
pub fn exec<'a, 'b>(&'a self, env: &'b mut Env<'a>) -> Result<Option<Vec<LuaValue>>, ASTExecError>
```

Note that `eval` returns the result of the vector of `LuaValue` because, in Lua, multiple values can be returned by expressions. The reason why `exec` returns the `Result` of the `Option` of a vector is to have a way to signal a `break` statement. When `None` is returned, it means the break statement has been called meaning that if the caller is one of `for`, `while`, or `repeat`, it should exit the loop.

Also, using immutable references means that we need to specify lifetime parameters. Note that the lifetime of `self` and `env` is different, because the lifetime of AST does not have a relationship with the lifetime of environment. However, we can see that `env` is taking in a lifetime of `self` as an argument (`(&'a self, env: &'b mut Env<'a>)`). This is because we need a lifetime parameter that can specify the lifetime of the function body when `LuaValue` is a function. Functions will have references to blocks that live inside AST, so the lifetime parameters that can represent the lifetime of AST are needed when we define `LuaValue` and `Env`. This was a tricky problem because if we link wrong lifetime parameters, the compiler will throw an error but the error messages are not always about the lifetimes, which may lead developers to incorrect directions. Having a good understanding of the relationship between these lifetimes was crucial to make sure the compiler understand the behavior correctly.

3. **Implementing Environment**

We had to change our implementation of the environment multiple times. Our first approach was having a `Vec<EnvTable>` as an environment where `EnvTable` is `Vec<(String, LuaValue)>`. However, if the name of the variable can only be String, using a `HashMap` is more efficient so we updated both `Vec`s to `HashMap`s. Since our Luavalue is defined as:

```Rust
struct LuaValue<'a>(Rc<LuaVal<'a>>);
```

multiple variables could be the owners of the same `LuaVal`.

However, we later noticed that Lua's functions can act as a closure. There were two possible approaches we could take from here: 1. Go through the closure body and identify the captured variables, 2. Capture the environment where closure is defined. First, we took the approach of identifying captured variables inside the closure's body. However, this increased the number of lines of code significantly and was not very efficient since we had to go through the entire function body even when it is not getting called. Also, it did not behave like Lua because when captured variables get updated, the entire `LuaValue` is overwritten inside the `Env`, not updating the `LuaVal` inside `Rc`. For example,

```Lua
local a = 2
function f()
    print(a)
end
a = a + 1
f()
```

should output `3`, but the explained design printed `2` because reassignment to environment overwrites the entire `LuaValue` inside the `HashMap` of the `EnvTable`, not the actual value inside the `Rc`. This means that captured variable `a` and the `a` outside of the function will be owning different values after line `a = a + 1`.

Our next approach was wrapping `Rc` and `RefCell` around `EnvTable` so that the closure can capture the scope where it is defined and mutate the captured scope. However, this brought a new problem in that we cannot differentiate between variables that are captured by the closure and the variables that are not captured by the closure if they are in the same scope. For example,

```Lua
local a = 1
function g()
    print(a)
    print(b)
end
local b = 3
g()
```

should print `1 nil`, but the explained design will print `1 3` because the captured scope can access variables that are defined in the scope even though they are defined after the closure definition. The solution to this problem was having multiple `EnvTable` to represent one scope instead of one `EnvTable` representing one scope. Our `LocalEnv` is defined as a vector of `Option` of `EnvTable`, and scopes will be divided with `None` inside the vector. For example, the local environment will look like `LocalEnv: [None, Some(EnvTable1), Some(EnvTable2), None, Some(EnvTable3)]` when `EnvTable1` and `EnvTable2` are in the same scope and `EnvTable3` is in different scope. More specifically, our definitions of environment will be the following:

```Rust
pub struct EnvTable<'a>(Rc<RefCell<HashMap<String, LuaValue<'a>>>>);

pub struct LocalEnv<'a>(Vec<Option<EnvTable<'a>>>);

pub struct Env<'a> {
    global: EnvTable<'a>,
    local: LocalEnv<'a>,
}
```

How does this help to solve our problem? We updated our `Block::exec` function as follows:

```Rust
fn exec<'a, 'b>(
        &'a self,
        env: &'b mut Env<'a>,
    ) -> Result<Option<Vec<LuaValue<'a>>>, ASTExecError> {
        let return_vals = self.exec_without_pop(env)?;
        // Remove environment when exiting a scope
        env.pop_local_env();
        // Add another table in the caller's scope
        // Avoiding local variable being inserted after closure is created
        env.extend_local_without_scope();

        Ok(return_vals)
    }
```

We call `env.extend_local_without_scope();` before we return to the caller so that the variables that are defined after the closure is defined in `EnvTable` that is not captured by the closure. For the example code above, the environment will look like this:

```
LocalEnv: [None, Some(EnvTable({'a': 1})), Some(EnvTable({'b': 3}))]
```

where closure captures `Some(EnvTable({'a': 1}))` but not `Some(EnvTable({'b': 3}))`, which will make printed value `1 nil`.

### Relevant aspects

1. **Testing**

We added multiple 92 unit tests and 33 integration tests to ensure that our parser and interpreter behave correctly.

```
$ cargo test -q

running 92 tests
........................................................................................ 88/92
....
test result: ok. 92 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 33 tests
.................................
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

2. **Comparison with official Lua**

Following is comparisons of the execution speed of MoonRust and the official Lua compiler. As a nature of the interpreter, we assumed our implementation will be slower than the compiler, but the speed difference was very huge. This shows that even though Rust is a very fast language, to build an efficient interpreter, developers have to spend a lot of time to figure out how to optimize each operation.

4829449
Calculating first 25 fibonacci numbers
**MoonRust**

```
$ cargo run -q assets/fibonacci.lua -s
Enter a number:
30
0
1
1
2
3
5
8
13
21
34
55
89
144
233
377
610
987
1597
2584
4181
6765
10946
17711
28657
46368
75025
121393
196418
317811
514229
832040

exec time   : 163.4942114000 seconds
```

**Official Lua**

```
TODO: Matt
```

Checking if 4829449 is prime

**MoonRust**

```
$ cargo run -q assets/prime_checker.lua -s
Enter a number:
4829449
4829449 is prime

exec time   : 34.2837148000 seconds
```

**Official Lua**

```
TODO: Matt
```

3. **Conclusion**

This project was quite challenging to finish in a couple of weeks during the semester, but our team was able to accomplish the goals of the project by implementing all the features of MVP. We had to change our design multiple times and throw away some of our implementations, but we practiced our skills in Rust and learned valuable lessons along the way (e.g. enhanced understanding of programming language design and parsing, Rust's lifetime). We were able to structure our project with modules and use advanced features like `Rc` and `RefCell` in multiple places. Overall, our team enjoyed working on a project with Rust because the code was very readable and the compiler catches simple logic errors that developers can easily miss. In the future, we would like to add syntactic sugars we skipped (for generic, literal string formats, etc.) and possibly implement more standard libraries.
