// TODO
// 6. Clean up code (delete unused lines, clippy code)
// 7. Final docs with demo prep (measure time)
use crate::interpreter::{LuaVal, LuaValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// One scope of bindings
#[derive(Debug, PartialEq, Clone)]
pub struct EnvTable<'a>(Rc<RefCell<HashMap<String, LuaValue<'a>>>>);
impl<'a> EnvTable<'a> {
    pub fn new() -> Self {
        EnvTable(Rc::new(RefCell::new(HashMap::new())))
    }

    pub fn get(&self, name: &str) -> Option<LuaValue<'a>> {
        let hm = self.0.borrow();
        let res = hm.get(name);
        res.map(|res| res.clone_rc())
    }

    pub fn get_mut(&mut self, name: &str) -> Option<LuaValue<'a>> {
        let mut hm = self.0.borrow_mut();
        let res = hm.get_mut(name);
        res.map(|res| res.clone_rc())
    }

    // Insert a new variable or update an existing one
    pub fn insert(&mut self, name: String, var: LuaValue<'a>) -> Option<LuaValue<'a>> {
        self.0.borrow_mut().insert(name, var)
    }
}

impl<'a> Default for EnvTable<'a> {
    fn default() -> Self {
        Self::new()
    }
}

// Insert None between each EnvTable to represent a new scope
#[derive(Debug, PartialEq)]
pub struct LocalEnv<'a>(Vec<Option<EnvTable<'a>>>);
impl<'a> LocalEnv<'a> {
    pub fn new() -> Self {
        let mut env = LocalEnv(vec![]);
        env.extend_env();
        env
    }

    pub fn get(&self, name: &str) -> Option<LuaValue<'a>> {
        // Start from top of the stack
        for table in self.0.iter().rev().flatten() {
            if let Some(var) = table.get(name) {
                return Some(var);
            }
        }
        None
    }

    // pub fn get_mut(&mut self, name: &str) -> Option<&mut LuaValue<'a>> {
    //     // Search in reversed order to check current scope first
    //     for table in self.0.iter_mut().rev() {
    //         for (var_name, var) in table.0.iter_mut() {
    //             if var_name == name {
    //                 return Some(var);
    //             }
    //         }
    //     }
    //     None
    // }

    pub fn extend_env(&mut self) {
        self.0.push(None);
        self.0.push(Some(EnvTable::new()));
    }

    pub fn extend_env_with_table(&mut self, table: &Option<EnvTable<'a>>) {
        match table {
            Some(table) => {
                self.0.push(Some(EnvTable(Rc::clone(&table.0))));
            }
            None => self.0.push(None),
        }
    }

    // None is inserted to represent a new scope
    // This function does not insert None
    pub fn extend_env_without_scope(&mut self) {
        self.0.push(Some(EnvTable::new()));
    }

    // Pop all tables in current scope
    pub fn pop_env(&mut self) {
        while let Some(Some(_)) = self.0.pop() {}
    }

    // Always inserting into the current scope
    pub fn insert(&mut self, name: String, var: LuaValue<'a>) -> Option<LuaValue<'a>> {
        match self.0.last_mut() {
            Some(table) => match table {
                Some(table) => table.insert(name, var),
                None => panic!("Environment is corrupted"),
            },
            None => panic!("Environment stack is empty"),
        }
    }

    pub fn update(&mut self, name: String, var: LuaValue<'a>) -> Option<LuaValue<'a>> {
        for table in self.0.iter_mut().rev().flatten() {
            if table.get(&name).is_some() {
                return table.insert(name, var);
            }
        }
        None
    }

    pub fn capture_env(&self) -> Self {
        let mut env = LocalEnv::new();
        for table in self.0.iter().rev() {
            env.extend_env_with_table(table);
        }
        env
    }
}

impl<'a> Default for LocalEnv<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq)]
pub struct Env<'a> {
    global: EnvTable<'a>,
    local: LocalEnv<'a>,
}

impl<'a> Env<'a> {
    pub fn new() -> Self {
        let mut env = Env {
            global: EnvTable::new(),
            local: LocalEnv::new(),
        };
        // Insert built-in functions
        env.insert_global("print".to_string(), LuaValue::new(LuaVal::Print));
        env.insert_global("read".to_string(), LuaValue::new(LuaVal::Read));
        env
    }

    // pub fn get_global_env(&self) -> &EnvTable<'a> {
    //     &self.global.borrow()
    // }

    pub fn set_global_env(&mut self, env: &Rc<RefCell<HashMap<String, LuaValue<'a>>>>) {
        self.global = EnvTable(Rc::clone(env));
    }

    pub fn get_local(&self, name: &str) -> Option<LuaValue<'a>> {
        self.local.get(name)
    }

    pub fn get_global(&self, name: &str) -> Option<LuaValue<'a>> {
        self.global.get(name)
    }

    pub fn get(&self, name: &str) -> Option<LuaValue<'a>> {
        self.local.get(name).or_else(|| self.global.get(name))
    }

    pub fn insert_local(&mut self, name: String, var: LuaValue<'a>) {
        self.local.insert(name, var);
    }

    pub fn update_local(&mut self, name: String, var: LuaValue<'a>) {
        self.local.update(name, var);
    }

    pub fn insert_global(&mut self, name: String, var: LuaValue<'a>) {
        self.global.insert(name, var);
    }

    // Only used for local environment
    pub fn extend_local_env(&mut self) {
        self.local.extend_env();
    }

    pub fn extend_local_without_scope(&mut self) {
        self.local.extend_env_without_scope();
    }

    // Only used for local environment
    pub fn pop_local_env(&mut self) {
        self.local.pop_env();
    }

    // Used for function (closures) capturing variables
    // TODO: probably not going to use it
    // pub fn vec_to_env(
    //     captured_vars: &Vec<(String, LuaValue<'a>)>,
    //     global_env: EnvTable<'a>,
    // ) -> Env<'a> {
    //     let mut new_env = Env::new();
    //     new_env.set_global_env(global_env);
    //     for (name, var) in captured_vars {
    //         new_env.insert_local(name.clone(), var.clone_rc());
    //     }
    //     new_env
    // }

    pub fn get_local_env(&self) -> &LocalEnv<'a> {
        &self.local
    }

    // Use captured local environment with current global environment
    pub fn create_with_captured_env(&self, local_env: &LocalEnv<'a>) -> Env<'a> {
        let mut new_env = Env::new();
        new_env.set_global_env(&self.global.0);
        // TODO: double check this
        new_env.local = local_env.capture_env();
        new_env
    }
}

impl<'a> Default for Env<'a> {
    fn default() -> Self {
        Self::new()
    }
}
