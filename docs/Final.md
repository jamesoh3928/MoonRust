# MoonRust

![alt text](../assets/moon.jpg)

MoonRust is Rust implementation of Lua interpreter. Lua means "moon" in Portuguese.

Team members:

- James Oh
- Matthew DellaNeve
- Renee Veit

## Summary Description

The goal of this project was to build an interpreter that will execute a subset of Lua given a file to read. The details of functionality are specified in the _Project Breakdown_ section. (Note: our implementation logic differs from Lua's reference implementation since we are not supporting the full features of Lua. Our interpreter should be able to run a simple Lua standalone program).

## Project Breakdown

-- TODO: delete placeholder --
Describe the work done for the project and lessons learned.

### Implementation

1. Parser (excluding syntactic sugar such as literal strings and table constructor)
2. Binary expression evaluation
3. Unary expressions evaluation
4. Control statement evaluation (if, else, break)
5. Loop statement evaluation (for, while, repeat)
6. Function definition/call evaluation
7. Visibility rules and scoping
8. Table evaluation

### Challenges/Lessons

1. Left recursive parsing was very tricky
2. First made eval/exec consume AST, but changed to take immutable reference in order to make function work
3. Lifetime parameters were tricky (Function will have reference to block which lives in AST, so the lifetime parameters will basically represent the lifetime of AST tokens, had to expand the lifetime parameters to many structs since lot of them are related, however, it was crucial to not link the lifetime of environment with AST, the lifetime parameter is for LuaValue stored in env, but that doesn't mean env also needs to have equal lifetime as AST) - immutable ref was needed because of function call and loops (need to re-evaluate the expressions)
4. Repeat until can refer to local variables in the loop for condition expression
5. Capturing variables/environment for closure..... Environment cannot be shared, traverse through the block
6. Lifetime errors....

```rust
match &*LuaValue::extract_first_return_val((*func).eval(env)?).0.borrow() {}
```

below code had "`rc` does not live long enough error", so had to make it one line

```rust
let func = LuaValue::extract_first_return_val((*func).eval(env)?);
let rc = func.0;
match &*rc.borrow() { ... }
```

## Additional Details

-- TODO: delete placeholder --

### List of dependencies

-nom = "7"
-clap = {version = "4.1", features = ["cargo", "derive"]}
-rand = "0.8.4"

### Structure of the code

- Briefly describe the structure of the code (what are the main components, the
  module dependency structure). Why was the project modularized in this way?

The project has three main components: the entrypoint, the parser, and the interpreter.
We decided to split it up this way because it nicely separates the stages of the program's execution. Additionally, the abstract syntax tree (AST) definition is separated out into its own section. We describe these components below:

#### Entrypoint

The entrypoint is contained within `main.rs`. It handles command-line arguments and retrieves the program text from the given file. Once it has this text, it passes it off to the parser stage, gets the resulting AST, and then passes that to the
interpreter stage for evaluation. It also benchmarks the execution time and displays that result if the user desires.

#### AST

The AST is contained entirely in the `ast.rs` file. It defines all of the data structures
that make up the AST that represents a Lua program. Described below are these data structures:

- `AST`: The root of the AST. This indicates where the Lua program starts. It contains a `Block` that represents the top-level statements in the program.
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
  - `Dot`: Represents fetching a value by accessing it from a table with a string key only. The table is stored behind a prefix expression.

Each data structure in the AST implements the `Display` and `Debug` traits. The `Display` trait was mostly useful for debugging any issues with the parser. The `Debug` trait is used to output the AST to the console after a successful parse.

#### Parser

Users interface with the parser through the `parser.rs` file. This file defines the public-facing code, including the top-level `parse` function that produces the AST representing the input program. The majority of the parser implementation is hidden behind various sub-modules in the `parser` folder. The `expression` and `statement` modules handle parsing components related to expressions and statements respectively. Each also has a public-facing function `parse_exp` and `parse_stmt` that return an `Expression` or `Statement` on a successful parse. The `common` module contains parsing functions that are useful for parsing sub-components of both expressions and statements. None of these functions are public-facing. The `util` module contains some utility parsing functions. These functions were borrowed or adapted from Nom's recipes (https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md).

The reason we decided to split up the structure of the parser this way is because expressions and statements are the dominating components of a Lua program. Most other syntactic forms are sub-components of expressions and statements, so we felt that organizing the parser accordingly made the most sense. Since expressions and statements share many of the same kinds of sub-components, we also decided that it would be natural to put those in a "common" file.

#### Interpreter

TODO

### Rusty code

- Choose (at least) one code excerpt that is a particularly good example of Rust
  features, idioms, and/or style and describe what makes it “Rusty”.

### Difficult to express in Rust

- Were any parts of the code particularly difficult to expres using Rust? What
  are the challenges in refining and/or refactoring this code to be a better
  example of idiomatic Rust?

### Different Approaches

- Describe any approaches attempted and then abandoned and the reasons why. What
  did you learn by undertaking this project?

When implementing the parser, we initially tried to write the parsing functions directly from the syntax specified in the Lua manual. However, this didn't work. Nom is a top-down parsing library, meaning it suffers from the same issue that all top-down parsers have: left recursion. Thus, we had several issues where running the parser would overflow the stack, since the parser would just infinitely expand some mutually recursive syntax rules. We solved this issue by changing our parsing strategy to factor out left recursion. For expressions, this was a matter of parsing according the operator precedence hierarchy. For prefix expressions, we parsed according to a "flattened" representation of the syntax in order to remove any ambiguity that was there in the original specification. One challenge there was that, since we were already using the AST to implement the interpreter, we could not change it build in this flattened prefix expression. Our workaround was to parse into an intermediate data structure, and then convert it back to our actual AST representation after a successful parse.

### Relevant aspects

- Review the final project grading rubric and discuss any relevant aspects of
  the project.
