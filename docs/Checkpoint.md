# MoonRust

Team members:

- James Oh
- Matthew DellaNeve
- Renee Veit

## Summary Description
**Reiterate the summary description of the overall goal of the project (updated as necessary from the Proposal document).**
Our goals stated in the proposal document are the following:

The goal of this project is to build an interpreter that will execute a subset of Lua given a file to read. The details of functionality can be found under the MVP section. (Note: our implementation logic might differ from Lua's reference implementation since we are not supporting the full features of Lua. Our interpreter should be able to run a simple Lua standalone program).

Our overall goal and MVP features of this project are still the same, but as we implement the parser and interpreter, we realized there are minor details of the language that we did not realize in the proposal phase, for instance, there are different ways to define literal strings in Lua. All of the strings denoted below have equal value in Lua.

```
a = 'alo\n123"'
a = "alo\n123\""
a = '\97lo\10\04923"'
a = [[alo
123"]]
a = [==[
alo
123"]==]
```   

There are many other syntactic sugars in Lua (such as table constructors and different ways to define float), and these are interesting problems. However, to fulfill our goal of this project (build a minimal Lua interpreter that can run a very simple Lua program), we decided to put these aside. As we continue working on the project, we anticipate setting these kinds of minor interesting problems aside and working on the main features, then coming back to them if we have time after finishing MVP.

## Checkpoint Progress Summary

**Give a summary of the progress made and lessons learned thus far.**

### 1. Parser - Expression
- **TODO: Matt**

### 2. Parser -  Statements
All of the Lua statements defined in our AST are implemented in `statement.rs`. The public `parse_stmt()` goes through each parse function and returns the result of the first successful parse. 

Each parse function needs to be individually tested before being integrated into the larger project. 

### 3. Interpreter
The big structure of the interpreter is defined in `interpreter.rs`. As the file gets larger, we might distribute code into multiple files. All of the structure for evaluating expressions, executing statements, and calling exec on `AST` is done. Semantics for basic variable assignments and numeral, string, and boolean expressions have been completed with unit tests added to the bottom of the file. 

An environment that will be used for scoping variables has been also implemented. Structs related to the environment are defined in `interpreter/environment.rs`.

For Lua variable assignment, if the number of variables exceeds the number of assigned values, the variables at the end are assigned with `nil` values. For example, for `a, b, c = 1, 2`, `c` will have `nil` assigned.

### 4. Other work
The main binary crate uses `clap` crate to parse the command line argument. The main binary is also capable of reading files now, and we just have to call the `parse` method on the string we read, and call `exec` on the `AST`.

```
$ cargo -q run -- --help
Usage: moonrust.exe <FILE.lua>

Arguments:
  <FILE.lua>  Path of the file to run

Options:
  -h, --help  Print help
```

### Some of the **lessons** we learned while implementing the interpreter.
- Rust ownership rules make developers think about data flows. However, sometimes it can be tough to do certain things (such as function calls I mentioned in the question below).
- Left recursive made it challenging to parse some expressions!
- Some tricks with visibility of modules (eg. declare `ast` module private, but declare `ast::AST` public)

## Additional Details
- List of dependencies added: 
[dependencies]
nom = "7"
clap = {version = "4.1", features = ["cargo", "derive"]}

- Structure of the code:
- main.rs: main binary
- lib.rs: main library crate
    - parser: **TODO: Matt/Renee**
      - common.rs
      - expression.rs
      - statement.rs
      - util.rs
    - parser.rs
    - interpreter
      - environment.rs: an environment for scoping variables
    - interpreter.rs: all the semantics of Lua

- Questions/Feedback
1. As we decided not to implement the standard library feature, it is hard to input or output our program to test our interpreter. We are thinking about adding `print` and `read` keywords just for testing purposes (not as a standard library, but we are going to parse them as keywords). We would love to hear some thoughts from the professor.
2. This question is related to semantics, and the related code can be found in `interpreter.rs`. Right now when we call `exec` or `eval` functions on any of the tokens in AST, we take self as an argument to consume all data when the execution is done. This logically makes more sense because we don't want to accidentally run the same line of code again. However, for functions, we want to save function data inside the environment and call the function multiple times when we want to. However, the current implementation of exec and eval will try to take ownership of data, which shouldn't be allowed function call (we should be able to call the function again later). There are two possible approaches we came up with First, we create `exec_mut` and `eval_mut` for all `exec` and `eval` method that exists, and those mut functions are only going to be called inside function calls. This may look redundant and increase the size of the code a lot. The second possible solution is implementing the `Clone` trait on all of the tokens on AST, and we can clone the function data from the environment and call exec and eval that will consume data. Both of the solutions will increase the size of the codebase by a lot. Do you have any suggestions or thoughts on this?
3. Prefix in Lua is defined as `prefixexp ::= var | functioncall | ‘(’ exp ‘)’`, and `functioncall` can return multiple values. This means that `prefixexp` can return multiple values and `exp` as a whole can return multiple values. Our current implementation enforces expressions to return `Result<LuaValue, ASTExecError>`, but it seems like we may need to change that to `Result<Vec<LuaValue>, ASTExecError>`. Do you think this is a reasonable solution, or are there any concerns regards to this solution? 
4. Do you have any feedback on the structure of our code?
5. We have faced many challenges until the checkpoint, and we are expecting to see more challenges as we continue working on the project. Do you still think we scoped this project realistically, or do you recommend scoping it down?