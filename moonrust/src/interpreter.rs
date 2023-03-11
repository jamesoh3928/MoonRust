// TODO: probably have to create files for each datatype inorder to make exec/eval functions for each of them
enum LuaValue {
    LuaTable(Table),
    LuaNil,
    LuaBool(bool),
    LuaNum([u8; 8]), // Represent numerals as an array of 8 bytes
    LuaString(String),
    Function(LuaFunction),
}

struct LuaFunction {
    name: String,
    arity: usize,
    /// The number of arguments
    statement: Vec<AST>,
}

struct LuaVar(Rc<RefCell<LuaValue>>);

 struct Table(Vec<(LuaValue, Rc<RefCell<LuaValue>>)>);
