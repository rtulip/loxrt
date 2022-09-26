use crate::interpreter::Types;
use std::collections::HashMap;

pub struct Environment {
    globals: HashMap<String, Types>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            globals: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Types) {
        self.globals.insert(name, value);
    }

    pub fn get(&self, name: &String) -> Option<&Types> {
        self.globals.get(name)
    }
}
