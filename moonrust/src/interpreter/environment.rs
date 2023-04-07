use crate::interpreter::LuaValue;
use std::{cell::RefCell, ops::{DerefMut, Deref}};
// TODO: double check environment implementation
// Dr. Fluet's advice: env: Vec<Table<String, Data>>, type Env = (Table<String, Data>, Vec<Table<String, Data>>)

// One scope of bindings
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
        // let mut i = 0;
        // for binding in self.0.borrow().iter() {
        //     if binding.0 == name {
        //         return Some(binding);
        //     }
        //     i += 1;
        // }
        // None
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
    pub fn insert(&'a mut self, name: String, var: LuaValue<'a>) {
        let name_slice = &name[..];
        for binding in self.0.iter_mut() {
            if binding.0 == name_slice {
                binding.1 = var;
                return;
            } 
        };
        self.0.push((name, var));

        // TODO: introduce RefCell again?
        // match self.get_mut(&name) {
        //     Some(original) => {
        //         // *original.0.borrow_mut().deref_mut() = var;
        //         (*original).1 = var;
        //     }
        //     None => {
        //         self.0.borrow_mut().push((name, var));
        //         // self.0.borrow_mut().push((name, var));
        //     }
        // };
    }
}

// Insert None between each EnvTable to represent a new scope
pub struct Env<'a>(Vec<EnvTable<'a>>);
impl<'a> Env<'a> {
    pub fn new() -> Self {
        let mut env = Env(vec![]);
        env.extend_env();
        env
    }

    pub fn get(&self, name: &str) -> Option<&LuaValue<'a>> {
        // Search in reversed order to check current scope first
        // let length = self.0.borrow().len();
        // for i in (0..length).rev() {
        //     let env = self.0.borrow();
        //     let env = env.deref();
        //     let table = &env[i] as *const EnvTable<'a>;
        //     let table: &EnvTable<'a> = unsafe { &*table };
        //     if let Some(var) = table.get(name) {
        //         let var = var as *const LuaValue;
        //         let var: &'a LuaValue = unsafe { &*var };
        //         return Some(var);
        //     }
        // }

        // Previous code that has error of "cannot return value referencing temporary value"
        for table in self.0.iter().rev() {
            return table.get(name);
        }
        None
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&LuaValue<'a>> {
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

    pub fn insert(&'a mut self, name: String, var: LuaValue<'a>) {
        match self.0.last_mut() {
            Some(table) => {
                table.insert(name, var);
            }
            None => panic!("Environment stack is empty"),
        };
    }
}
