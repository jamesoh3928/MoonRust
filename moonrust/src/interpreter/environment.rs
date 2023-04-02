use crate::interpreter::LuaVar;
// TODO: double check environment implementation
// Dr. Fluet's advice: env: Vec<Table<String, Data>>, type Env = (Table<String, Data>, Vec<Table<String, Data>>)

// One scope of bindings
pub struct EnvTable(Vec<(String, LuaVar)>);
impl EnvTable {
    pub fn new() -> Self {
        EnvTable(vec![])
    }

    pub fn get(&self, name: &str) -> Option<&LuaVar> {
        for (var_name, var) in self.0.iter() {
            if var_name == name {
                return Some(var);
            }
        }
        None
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut LuaVar> {
        for (var_name, var) in self.0.iter_mut() {
            if var_name == name {
                return Some(var);
            }
        }
        None
    }

    // Insert a new variable or update an existing one
    pub fn insert(&mut self, name: String, var: LuaVar) {
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
pub struct Env(Vec<Option<EnvTable>>);
impl Env {
    pub fn new() -> Self {
        Env(vec![])
    }

    pub fn get(&self, name: &str) -> Option<&LuaVar> {
        // Search in reversed order to check current scope first
        for scope in self.0.iter().rev() {
            match scope {
                Some(EnvTable(table)) => {
                    for (var_name, var) in table.iter() {
                        if var_name == name {
                            return Some(var);
                        }
                    }
                }
                None => (),
            }
        }
        None
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut LuaVar> {
        // Search in reversed order to check current scope first
        for scope in self.0.iter_mut().rev() {
            match scope {
                Some(EnvTable(table)) => {
                    for (var_name, var) in table.iter_mut() {
                        if var_name == name {
                            return Some(var);
                        }
                    }
                }
                None => (),
            }
        }
        None
    }

    pub fn extend_env(&mut self) {
        self.0.push(None);
        self.0.push(Some(EnvTable::new()));
    }
}
