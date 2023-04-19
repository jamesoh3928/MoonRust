use crate::interpreter::LuaValue;
// TODO: double check environment implementation
// Dr. Fluet's advice: env: Vec<Table<String, Data>>, type Env = (Table<String, Data>, Vec<Table<String, Data>>)

// One scope of bindings
#[derive(Debug, PartialEq)]
pub struct EnvTable<'a>(Vec<(String, LuaValue<'a>)>);
impl <'a>EnvTable<'a> {
    pub fn new() -> Self {
        EnvTable(vec![])
    }

    pub fn get(&self, name: &str) -> Option<& LuaValue<'a>> {
        for (var_name, var) in self.0.iter() {
            if var_name == name {
                return Some(var);
            }
        }
        None
    }

    // pub fn get_mut(&'a mut self, name: &str) -> Option<&mut (String, LuaValue<'a>)> {
    //     let mut i = 0;
    //     for (var_name, var) in self.0.borrow().iter() {
    //         if var_name == name {
    //             return self.0.borrow_mut().get_mut(i);
    //         }
    //         i += 1;
    //     }
    //     None
    // }

    // Insert a new variable or update an existing one
    pub fn insert(&mut self, name: String, var: LuaValue<'a>) {
        let name_slice = &name[..];
        for binding in self.0.iter_mut() {
            if binding.0 == name_slice {
                binding.1 = var;
                return;
            } 
        };
        self.0.push((name, var));
    }
}

// Insert None between each EnvTable to represent a new scope
#[derive(Debug, PartialEq)]
pub struct LocalEnv<'a>(Vec<EnvTable<'a>>);
impl<'a> LocalEnv<'a> {
    pub fn new() -> Self {
        let mut env = LocalEnv(vec![]);
        env.extend_env();
        env
    }

    pub fn get(&self, name: &str) -> Option<&LuaValue<'a>> {
        // Previous code that has error of "cannot return value referencing temporary value"
        for table in self.0.iter().rev() {
            if let Some(var) = table.get(name) {
                return Some(var);
            }
        }
        None
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut LuaValue<'a>> {
        // Search in reversed order to check current scope first
        for table in self.0.iter_mut().rev() {
            for (var_name, var) in table.0.iter_mut() {
                if var_name == name {
                    return Some(var);
                }
            }
        }
        None
    }

    pub fn extend_env(&mut self) {
        self.0.push(EnvTable::new());
    }

    pub fn pop_env(&mut self) -> EnvTable<'a> {
        match self.0.pop() {
            Some(env) => env,
            None => panic!("Environment stack is empty"),
        }
    }

    pub fn insert(&mut self, name: String, var: LuaValue<'a>) {
        match self.0.last_mut() {
            Some(table) => {
                table.insert(name, var);
            }
            None => panic!("Environment stack is empty"),
        };
    }
}

#[derive(Debug, PartialEq)]
pub struct Env<'a> {
    global: EnvTable<'a>,
    local: LocalEnv<'a>,
}

impl<'a> Env<'a> {
    pub fn new() -> Self {
        Env {
            global: EnvTable::new(),
            local: LocalEnv::new(),
        }
    }

    pub fn get_local(&self, name: &str) -> Option<&LuaValue<'a>> {
        self.local.get(name)
    }

    pub fn get_global(&self, name: &str) -> Option<&LuaValue<'a>> {
        self.global.get(name)
    }

    pub fn get(&self, name: &str) -> Option<&LuaValue<'a>> {
        self.get_local(name).or_else(|| self.get_global(name))
    }

    pub fn insert_local(&mut self, name: String, var: LuaValue<'a>) {
        self.local.insert(name, var);
    }

    pub fn insert_global(&mut self, name: String, var: LuaValue<'a>) {
        self.global.insert(name, var);
    }

    // Only used for local environment
    pub fn extend_local_env(&mut self) {
        self.local.extend_env();
    }

    // Only used for local environment
    pub fn pop_local_env(&mut self) -> EnvTable<'a> {
        self.local.pop_env()
    }

    // TODO: maybe mutable get?
}