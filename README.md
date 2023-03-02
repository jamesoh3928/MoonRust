# MoonRust
![alt text](assets/moon.jpg)
MoonRust is Rust implementation of Lua interpreter. Lua means "moon" in Portuguese.


## What is Lua?
Lua is a robust, fast, and lightweight scripting language that can be used on different platforms. It is widely used in embedded systems and games because of its performance. Lua’s interpreter can be compiled with ANSI C compiler (standardized C since 1989). 

Use cases: Roblox, Adobe Photoshop, World of Warcraft, Angry Birds

Official docs: https://www.lua.org/ 

## Why build a Lua interpreter with Rust?
Well, we are taking a programming language course after all, so our main purpose is to enhance our understanding of Rust. We also wanted to build something that can appeal to Rust's strengths (memory safety and high performance). If an interpreter is keep causing memory issues and is slow, no developer would want to use it!

We wanted to scope the project so that it's doable in one semester. Lua is very lightweight (official C implementation only contains 30000), but a language that is widely used. If you check [Lua wiki](http://lua-users.org/wiki/LuaImplementations) page, there are many different implementations of Lua, but Rust is still missing! At this point, we decided to create a Lua interpreter in Rust! Also, it will be super fun to create your interpreter :).

## Proposal
Following is our proposal for this project.
### Goal
The goal of this project is to build a Lua interpreter that will be interacted with REPL. The details of functionality can be found under the MVP section.

### Use Cases
Users will able to interact with the program by running REPL (Read-Eval-Print Loop). When the `cargo -q run` command is entered, REPL will start and users will be able to write the Lua program they want to execute.

### Intended Components
@Matt I need your help on this one

### Testing
@Matt I also need help on this lol

### Minimum Viable Product
**Pasrsing/Lexsing**:

[Keywords in Lua](https://www.lua.org/manual/5.1/manual.html#8:~:text=The%20following-,keywords,-are%20reserved%20and)

We will implement the full syntax of Lua specified in [Lua's Reference Manual](https://www.lua.org/manual/5.1/manual.html#8)
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
[Official Lua Semantics](https://www.lua.org/manual/5.1/manual.html#2)
1. Values and Types
There are 8 basic types in Lua:  nil, boolean, number, string, function, userdata, thread, and table, but we are going to implement 6 of them excluding userdata and thread.

2. Variables
Users will be able to assign variables.

3. Statements
Chunks, Blocks, Assignment, Control Structures, For Statement, Function Calls as Statements, Local Declarations will be implemented.

4. Expressions
Arithmetic Operators, Relational Operators, Logical Operators, Concatenation, The Length Operator, Precedence, Table Constructors, Function Calls, Function Definitions will be implemented. 

5. Visibility Rules
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
1. Lua allows shared state unlike Rust's ownership rule
2. None of our teammates know Lua so a learning curve is expected


### Stretch Goals
1. Implement userdata, thread
2. Garabage collector
3. Environments
4. Metatables
5. Coroutines

### Expected Functionality By Checkpoint
By checkpoint, we are expecting to finish 
## Team members:
- James Oh
- Matthew DellaNeve
- Renee Veit


# TODO #
1. Check the scope of the project (what do you guys think?)