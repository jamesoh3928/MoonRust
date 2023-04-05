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
- **TODO: Renee**

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
2. Do you have any feedback on the structure of our code?
3. We have faced many challenges until the checkpoint, and we are expecting to see more challenges as we continue working on the project. Do you still think we scoped this project realistically, or do you recommend scoping it down?