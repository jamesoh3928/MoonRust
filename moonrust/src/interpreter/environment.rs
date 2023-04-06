use crate::interpreter::LuaValue;
// TODO: double check environment implementation
// Dr. Fluet's advice: env: Vec<Table<String, Data>>, type Env = (Table<String, Data>, Vec<Table<String, Data>>)

// One scope of bindings
pub struct EnvTable(Vec<(String, LuaValue)>);
impl EnvTable {
    pub fn new() -> Self {
        EnvTable(vec![])
    }

    pub fn get(&self, name: &str) -> Option<&LuaValue> {
        for (var_name, var) in self.0.iter() {
            if var_name == name {
                return Some(var);
            }
        }
        None
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut LuaValue> {
        for (var_name, var) in self.0.iter_mut() {
            if var_name == name {
                return Some(var);
            }
        }
        None
    }

    // Insert a new variable or update an existing one
    pub fn insert(&mut self, name: String, var: LuaValue) {
        match self.get_mut(&name) {
            Some(original) => {
                *original = var;
            }
            None => {
                self.0.push((name, var));
            }
        }
    }
}

// Insert None between each EnvTable to represent a new scope
pub struct Env(Vec<EnvTable>);
impl Env {
    pub fn new() -> Self {
        let mut env = Env(vec![]);
        env.extend_env();
        env
    }

    pub fn get(&self, name: &str) -> Option<&LuaValue> {
        // Search in reversed order to check current scope first
        for table in self.0.iter().rev() {
            for (var_name, var) in table.0.iter() {
                if var_name == name {
                    return Some(var);
                }
            }
        }
        None
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut LuaValue> {
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

    pub fn pop_env(&mut self) -> EnvTable{
        match self.0.pop() {
            Some(env) => env,
            None => panic!("Environment stack is empty"),
        }
    }

    pub fn insert(&mut self, name: String, var: LuaValue) {
        match self.0.last_mut() {
            Some(table) => {
                table.insert(name, var);
            }
            None => panic!("Environment stack is empty"),
        };
    }
}
