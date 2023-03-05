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

The goal of this project is to build a Lua interpreter that will be interacted with REPL. The details of functionality can be found under the MVP section.

### Use Cases

Users will able to run Lua program by specifiying Lua file. When the `cargo -q run [filename]` command is entered, interpreter will execute the code inside the file.

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

<!-- TODO: Renee one more usecase (maybe little more lua specific code - have block and if statment). Also, make sure we specify users to input the file. -->

### Intended Components

The three parts of interpreting (tokenizing, parsing, and execution) into their own separate modules. Each module will have their own main function for tokenizing, parsing, and execution respectively, as well as additional helper methods for various checks on the input. 

#### **Parser**

The parser takes the input program as a string and produces an
Abstract Syntax Tree (AST). Ours will be built on top of
[Nom](https://github.com/rust-bakery/nom), a parser combinator library
that provides some essential building-block functions for parsing
small components of the input.

Our parser will consist of specialized functions that parse individual
components of the input. These functions will roughly correspond to the
pieces of syntax defined in the Lua Reference Manual. The signature of
each function will look something like:

```rust
fn parse_syntax(input: &str) -> IResult<&str, AST, ParseErr> {...}
```

#### **Data Types**
We are going to implement 6 different Lua data types specified in the MVP section with the following `enum`.
```rust
Enum LValue {
   LTable(table: Table),
   Nil,
   LBool(b: bool),
   LNum(n: usize),
   LString(s: String), 
   Function(f: fn),
}
```

#### Options for Table:
For the `table` type, we are currently considering two different approaches. One is storing a vector that contains a tuple of name and corresponding Lua value, and we would just iterate through the vector and find a tuple matching the first value in the tuple. Note that the type of "name" of the value stored in the table is `LValue` because, in Lua, any kind of expression can be key in the table. This option will be a lot cleaner to implement, but the performance of accessing an element in the table will be inefficient.
```rust
Table(Vec<(LValue, Rc<LValue>>))
```

The second option is using a `HashTable` where the key is `LValue` and the value will be `Rc<LValue>`. This will make accessing elements in the table faster, but we might run into some obstacles in making `LValue` hashable.
```rust
Struct Table(t: HashMap<LValue, Rc<LValue>>)
```

TODO (James): might have to use this option, if not delete them
```rust
Struct Table {
   strTable: HashMap<LString, Rc<LValue>>,
   boolTable: HashMap<LBool, Rc<LValue>>,
   ....
}
```

**Variable**
The variable will be pointing at `LValue`, and since Lua allows multiple variables owning the same values, we will wrap this with `Rc`. Also, since the new type can be defined on the new variable, we would have to use trait object.
```rust
struct LVar(Rc<dyn LValue>)
```

** Function **
```rust
struct LFunction {
   name: String,
   arguments: Vec<dyn DataType>,
   statement: Vec<AST>,
}
```
The `eval` method of `LFunction` will return `Vec<dyn DataType>` since Lua allows functions to return different types of values.

<!-- TODO (Matt): following BNF grammar -->
```
enum AST {
   Variable(name: String, value: data_types),
   Block(...)
   Chunk(...)
   .....,
}
```
The AST type will be an `enum` where each variant represents a piece
of syntax. Each variant can also have data associated with it. For
example, a `Number` variant would hold that value of that number.
Additionally, Since pieces of syntax can contain other sub-pieces of
syntax, a variant may hold a `Box<AST>`.

#### **Semantics: Evaluation/Execution**
We defined the semantics that we are going to implement in the MVP section. All expressions will implement their own `eval` methods. For example, `LBool` will have the following `eval` method.
```rust
fn eval(&self) -> LBool {
   self.b
}
```
According to BNF grammar from Lua's reference manual, following are possible expressions:

```
nil | false | true | Numeral | LiteralString | ‘...’ | functiondef | prefixexp | tableconstructor | exp binop exp | unop exp 
```

For statements, we will have `exec` methods. Since the statment will be executed exactly once, the `exec` method will take ownership of `self`.
```rust
fn exec(self) -> LValue {
   ...
}
```

**Control Structures**
The control structure (if statement) will contain the following data.
```rust
struct Control {
   exps: Vec<exp>,
   blocks: Vec<block>,
   else_block: Option<block>
}
```

### Testing
<!-- TODO: Renee -->
Test `parse_syntax`
Test all `eval` methods for expressions and `exec` methods for statments.

We might consider using `test_case`

### Minimum Viable Product

**Pasrsing/Lexsing**:

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
3. Control statements: Lua's false rule
4. Implementing a table constructor might be challenging since there are many different ways to specify key and value for Lua's table.

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
