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
  - `FunctionCall`:

#### Parser

Users interface with the parser through the `parser.rs` file. This file defines

#### Interpreter

### Rusty code

- Choose (at least) one code excerpt that is a particularly good example of Rust
  features, idioms, and/or style and describe what makes it “Rusty”.

### Difficult to express in Rust

- Were any parts of the code particularly difficult to expres using Rust? What
  are the challenges in refining and/or refactoring this code to be a better
  example of idiomatic Rust?

### Different approaches

- Describe any approaches attempted and then abandoned and the reasons why. What
  did you learn by undertaking this project?

### Relevant aspects

- Review the final project grading rubric and discuss any relevant aspects of
  the project.
