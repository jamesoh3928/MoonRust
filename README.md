# MoonRust

![alt text](assets/moon.jpg)

MoonRust is Rust implementation of Lua interpreter. Lua means "moon" in Portuguese.

## What is Lua?

Lua is a robust, fast, lightweight, and cross-platform scripting language
designed to be embedded into other applications. It accomplishes this by
providing an API allowing C applications to interface it with. It's fairly minimal as well, making it easy to learn, but it's also fairly extensible
for cases where more advanced features are needed. For these reasons,
Lua has become a popular choice for video game programmers and is
natively supported by Roblox, Garry's Mod, World of Warcraft, and more.

Official docs: https://www.lua.org/

## Why build a Lua interpreter with Rust?

We have two main motivations for why we decided on this project. First,
we want to demonstrate our understanding of Rust by building a practical,
real-world application. Second, compilers and interpreters sit at the
heart of every programming language, and it's critical that they are
fast and memory-safe. Otherwise, programmers would become rightfully
annoyed with slow compile times and hidden bugs that have nothing to do
with their programs. Thus, Rust feels like the natural choice to build
an interpreter. Additionally, we find compilers and interpreters to be a
fascinating discipline. This project presents an opportunity to learn the
topic hands on.

We wanted to scope the project so that it's doable in one semester. Lua is very small (official C implementation only contains 30000 lines), but widely used. The [Lua wiki](http://lua-users.org/wiki/LuaImplementations) page shows there are many different implementations of Lua, but none in Rust. For these reasons, Lua felt like the right choice for a source
language.

## Proposal

The following is our proposal for this project.

### Goal

The goal of this project is to build an interpreter that will execute a subset of Lua given a file to read. The details of functionality can be found under the MVP section.

### Use Cases

Users will able to run Lua programs by specifiying a Lua file. When the `cargo -q run [filename]` command is entered, the interpreter will execute the code inside the file.

For example, say a user wanted to calculate the area and perimeter of an equilateral triangle. They would read the following code from the file:

```
print("Find the area of an equilateral triangle:")

-- Define variables
local height, base, area

-- Initialize variables
base = 10
height = 8.66

-- Basic Arithmetic
area = (base * height) / 2

-- Print Result
print("Area of triangle: ", area)
```

Then our program would output the following result:

```
Find the area of an equilateral triangle:
Area of triangle: 	43.3
```

Or if a user wanted to find the maximum and minimum two numbers, the following code would be read in from the file:

```
function min_and_max(num1, num2)
  --- block 1: find max value
  do
    if (num1 > num2) then
      result = num1;
   else
      result = num2;
   end
    print("Max value = ", result)
  end
  --- block 2: find min value
  do
    if (num1 < num2) then
      result = num1;
   else
      result = num2;
   end
    print("Min value = ", result)
  end
end

min_and_max(4, 11)
```

Then our program would output the following:

```
Output:

Max value = 	11
Min value = 	4
```

<!-- TODO: Renee one more usecase (maybe little more lua specific code - have block and if statment). Also, make sure we specify users to input the file. -->

### Intended Components

The project will consist of two library crates, the parser and the evaluator, and a binary crate that acts as the entry point to the program.

#### **Parser**

The parser takes the input program as a string and produces an Abstract Syntax Tree (AST). Ours will
be built on top of [Nom](https://github.com/rust-bakery/nom), a parser combinator library
that provides some essential building-block functions for parsing small components of the input. We
can use these to write our own parsing functions for parts of Lua and build those up into a parser for
the whole language (or at least what we will be implementing). These functions will roughly
correspond to the pieces of syntax defined in the Lua Reference Manual. The signature of
each function will look something like

```rust
fn parse_syntax(input: &str) -> IResult<&str, AST, ParseErr> {...}
```

where `input` is the current text to parse, and `IResult` is Nom's wrapper around a `Result`
that, if the parse is successful, will return `Ok` of the input that wasn't consumed and an
abstract syntax tree (described in the next section). If the parse fails, the function returns
`Err` with a `ParseErr`.

#### **Data Types**

We are going to implement 6 different Lua data types specified in the MVP section with the following `enum`.

```rust
enum LuaValue {
   LTable(Table),
   Nil,
   LBool(bool),
   LNum(usize),
   LString(String),
   Function(fn),
}
```

#### Options for Table:

For the `table` type, we are currently considering two different approaches. One is storing a vector that contains a pair of a key and corresponding Lua value. To get or update a value at a given key, we would iterate through the vector and find the pair where the first element is the key. Note that the type of a key stored in the table is an `LValue` because in Lua, any value (barring `nil`) can be a key in the table. This option will be a lot cleaner to implement, but it will have poor
performance compared to a standard key-value collection.

```rust
struct Table(Vec<(LuaValue, Rc<LuaValue>>))
```

The second option is using a `HashMap` where the key is `LValue` and the value will be `Rc<LuaValue>`. This will make accessing elements in the table faster, but we might run into some obstacles in making `LuaValue` hashable (for example, floating point values do not implement `Hash` by default in Rust).

```rust
struct Table(HashMap<LuaValue, Rc<LuaValue>>)
```

```rust
struct Table {
   strTable: HashMap<String, Rc<LuaValue>>,
   boolTable: HashMap<bool, Rc<LuaValue>>,
   ....
}
```

**Variable**

A variable will point to a `LuaValue`, and since Lua allows multiple variables to point at the
same piece of data and potentially modify it, we will wrap this with `Rc` and `RefCell`.

```rust
struct LuaVar(Rc<LuaValue>)
```

**Function**

Functions are represented by their name, the number of arguments it takes, and the statements
to execute when the function is called.

(NOTE: I renamed the arguments field to arity, and I changed the type to usize because we don't
actually care about the type of the arguments until we perform an operation that needs the type)

```rust
struct LuaFunction {
   name: String,
   arity: usize, /// The number of arguments
   statement: Vec<AST>,
}
```

The `eval` method of `LFunction` will return `Vec<DataType>` since Lua allows functions to return multiple values.

**The Abstract Syntax Tree (AST)**

The AST is how a program will be represented after parsing has completed. The various statements
and expressions in Lua are represented by the `Statement` and `Expression` enums. Additionally,
the binary and unary operations are enumerated as `BinOp` and `UnOp`. A block is represented
as a struct holding a vector of `Statement`s and an optional return statement. At the top level,
the `AST` struct simply holds a `Block`. This means that Lua programs are contained within
one root block. In the real implementation of Lua, this is not the whole story since it has
chunks and modules that complicate this notion. For our purposes, we only expect a Lua program to
be one block in a single file. Listed below are the relevant types for the AST:

```rust
enum Statement {
   Semicolon,
   Assignment((Vec<Var>, Vec<Expression>)),
   FunctionCall((PrefixExp, Option<String>)),
   Break,
   DoBlock(Block),
   While((Expression, Block)),
   Repeat((Block, Expression)),
   If((Expression, Block, Vec<(Expression, Block)>, Option<Block>)),
   ForNum((String, i64, i64, Option<i64>)),
   ForGeneric((Vec<String>, Vec<Expression>, Block)),
   FunctionDecl((String, ParList, Block)),
   LocalFuncDecl((String, ParList, Block))
}
```

```rust
enum Expression {
   Nil,
   False,
   True,
   NumeralInt(i64),
   NumeralFloat(f64),
   LiteralString(String),
   DotDotDot, /// Used for a variable number of arguments in things like functions
   FunctionDef((ParList, Block)),
   PrefixExp(PrefixExp),
   TableConstructor(Vec<Field>),
   BinaryOp((Expression, BinOp, Expression)),
   UnaryOp((UnOp, Expression))
}
```

```rust
enum BinOp {
   Add,
   Sub,
   Mult,
   Div,
   IntegerDiv,
   Pow,
   Mod,
   BitAnd,
   BitXor,
   BitOr,
   ShiftRight,
   ShiftLeft,
   Concat,
   LessThan,
   LessEq,
   GreaterThan,
   GreaterEq,
   Equal,
   NotEqual,
   LogicalAnd,
   LogicalOr
}
```

```rust
enum UnOp {
   Negate,
   LogicalNot,
   Length,
   BitNot
}
```

```rust
enum PrefixExp {
   Var(Var),
   FunctionCall(...),
   Exp(Expression)
}
```

```rust
struct ParList(Vec<String>, bool) // boolean flag is true if there are varargs
```

```rust
enum Field {
   BracketedAssign((Expression, Expression)),
   NameAssign((String, Expression)),
   UnnamedAssign(Expression)
}
```

```rust
enum Var {
   NameVar(String),
   BracketVar((PrefixExp, Exp)),
   DotVar((PrefixExp, String))
}
```

```rust
struct Block {
   statements: Vec<Statement>,
   return_stat: Option<Vec<Expression>>
}
```

```rust
struct AST(Block)
```

#### **Semantics: Evaluation/Execution**

The implementation of an `AST` will consist of an `eval` method that executed the code inside
the top-level block. Since most of the work will be delegated to the `Block` struct, this method
will be very simple:

```rust
impl AST {
   pub fn eval(&self) {
      self.0.eval();
   }
}
```

The `Block` struct will implement its own `eval` method. It will step through each statement and
execute them. Additionally, it will manage the data currently in its scope as it executes. If the
block has a return statement, it will evaluate the expressions inside of it and return the result
as the return value of `eval`.

`Statement` and `Expression` will also have their own `eval` methods. However, since each has a
large number of variants, most of the work will be delegated to helper methods for each variant
(e.g. `eval_assignment`, `eval_binary_op`, ...). The main `eval` method of each will simply
pattern match on each variant and call the appropriate function.

### Testing

1. The Lexer (Tokenizing Step)
   We will create an exhaustive test suite to make sure the input is tokenized according to the language grammar. Tokens will be mapped to a specific token type, and we will make sure the tokenizer recognizes all possible variations of a type. Here is example test suite for local and global variables in Lua:

```
local d , f = 2 ,9     --declaration of d and f as local variables.
d , f = 4, 1;          --declaration of d and f as global variables.
d, f = 17              --[[declaration of d and f as global variables.
                           Here value of f is nil --]]
10 = 20                -- error
```

2. The Parser (AST Creation)
   To test the parser (i.e. the `parse_syntax`, `eval`, and `exec` methods) we will compare the parser generated AST to a predefined AST representing the expected output. Expressions will be tested with the `eval` methods and statements will be tested with the `exec` methods.

3. The Interpreter (Execution)
   To test the interpreter we will make sure the commands executed using the AST produce the desired output.

We will make sure each stage rejects bad inputs and reports specific error messages describing why. We might consider using `test_case`

### Minimum Viable Product

**Pasrsing**:

[Keywords in Lua](https://www.lua.org/manual/5.4/manual.html#8:~:text=The%20following-,keywords,-are%20reserved%20and)

We will implement the full syntax of Lua specified in [Lua's Reference Manual](https://www.lua.org/manual/5.4/manual.html#8)

```
chunk ::= {stat [`;´]} [laststat [`;´]]

block ::= chunk

stat ::=  varlist `=´ explist |
        functioncall |
        do block end |
        while exp do block end |
        repeat block until exp |
        if exp then block {elseif exp then block} [else block] end |
        for Name `=´ exp `,´ exp [`,´ exp] do block end |
        for namelist in explist do block end |
        function funcname funcbody |
        local function Name funcbody |
        local namelist [`=´ explist]

laststat ::= return [explist] | break

funcname ::= Name {`.´ Name} [`:´ Name]

varlist ::= var {`,´ var}

var ::=  Name | prefixexp `[´ exp `]´ | prefixexp `.´ Name

namelist ::= Name {`,´ Name}

explist ::= {exp `,´} exp

exp ::=  nil | false | true | Number | String | `...´ | function |
        prefixexp | tableconstructor | exp binop exp | unop exp

prefixexp ::= var | functioncall | `(´ exp `)´

functioncall ::=  prefixexp args | prefixexp `:´ Name args

args ::=  `(´ [explist] `)´ | tableconstructor | String

function ::= function funcbody

funcbody ::= `(´ [parlist] `)´ block end

parlist ::= namelist [`,´ `...´] | `...´

tableconstructor ::= `{´ [fieldlist] `}´

fieldlist ::= field {fieldsep field} [fieldsep]

field ::= `[´ exp `]´ `=´ exp | Name `=´ exp | exp

fieldsep ::= `,´ | `;´

binop ::= `+´ | `-´ | `*´ | `/´ | `^´ | `%´ | `..´ |
        `<´ | `<=´ | `>´ | `>=´ | `==´ | `~=´ |
        and | or

unop ::= `-´ | not | `#´
```

**Semantics**:
[Official Lua Semantics](https://www.lua.org/manual/5.4/manual.html#2)

1. Values and Types:
   There are 8 basic types in Lua: nil, boolean, number, string, function, userdata, thread, and table, but we are going to implement 6 of them **excluding userdata and thread**.

2. Variables:
   Users will be able to assign variables.

3. Statements:
   Chunks, Blocks, Assignment, Control Structures, For Statement, Function Calls as Statements, Local Declarations will be implemented **excluding chunks (loading external sources)**.

4. Expressions:
   Arithmetic Operators, Relational Operators, Logical Operators, Concatenation, The Length Operator, Precedence, Table Constructors, Function Calls, Function Definitions will be implemented.

5. Visibility Rules:
   Lua is a lexically scoped language, and our interpreter will follow the visibility rules of Lua. Example from Lua's reference manual:

```
x = 10                -- global variable
do                    -- new block
    local x = x         -- new 'x', with value 10
    print(x)            --> 10
    x = x+1
    do                  -- another block
        local x = x+1     -- another 'x'
        print(x)          --> 12
    end
    print(x)            --> 11
end
print(x)              --> 10  (the global one)
```

### Expected Challenges

1. Lua allows shared state unlike Rust's ownership rule (We will be using a lot of `Rc`s)
2. None of our teammates know Lua so a learning curve is expected
3. Control statements: Lua treats everything that is not `nil` or `false` as true when evaluating
   conditions.
4. Implementing a table constructor might be challenging since there are many different ways to specify a key and value for Lua's table. In other words, Lua's flexibility makes things harder
   for us to implement.

### Stretch Goals

1. Implement chunks, userdata, thread
2. Garabage collector
3. Environments
4. Metatables
5. Coroutines
6. Other use cases: REPL, etc

### Expected Functionality By Checkpoint

By the checkpoint, a fully functional parser should be implemented, and "Values and Types" and "Variable" from the semantics section should be functional as well.

## Team members:

- James Oh
- Matthew DellaNeve
- Renee Veit
