use crate::error::LoxError;
use crate::interpreter::Types;
use crate::tokens::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    pub values: HashMap<String, Types>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            parent: None,
            values: HashMap::new(),
        }))
    }

    pub fn new_child(parent: &Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            parent: Some(parent.clone()),
            values: HashMap::new(),
        }))
    }

    pub fn get_parent(this: &Rc<RefCell<Self>>, depth: usize) -> Rc<RefCell<Self>> {
        if depth == 0 {
            return this.clone();
        } else {
            Environment::get_parent(&this.borrow().parent.as_ref().unwrap(), depth - 1)
        }
    }

    pub fn define(&mut self, name: String, value: Types) {
        self.values.insert(name, value);
    }

    pub fn get(&self, token: &Token) -> Result<Types, LoxError> {
        if self.values.contains_key(&token.lexeme) {
            Ok(self.values.get(&token.lexeme).unwrap().clone())
        } else if self.parent.is_some() {
            self.parent.as_ref().unwrap().borrow().get(token)
        } else {
            LoxError::new_runtime(
                token.line,
                format!("Failed to get undefined variable `{}`.", token.lexeme),
            )
        }
    }

    pub fn get_at(&self, token: &Token, depth: usize) -> Result<Types, LoxError> {
        if depth == 0 {
            self.get(token)
        } else {
            if let Some(parent) = &self.parent {
                parent.borrow().get_at(token, depth - 1)
            } else {
                LoxError::new_runtime(
                    token.line,
                    format!("Bad depth. Looking for depth {depth}, but no parent found."),
                )
            }
        }
    }

    pub fn set(&mut self, token: &Token, value: Types) -> Result<(), LoxError> {
        if self.values.contains_key(&token.lexeme) {
            *self.values.get_mut(&token.lexeme).unwrap() = value;
            Ok(())
        } else if self.parent.is_some() {
            self.parent
                .as_mut()
                .unwrap()
                .borrow_mut()
                .set(token, value)?;
            Ok(())
        } else {
            LoxError::new_runtime(
                token.line,
                format!("Failed to set undefined variable: `{}`.", token.lexeme),
            )
        }
    }

    pub fn set_at(&mut self, token: &Token, value: Types, depth: usize) -> Result<(), LoxError> {
        if depth == 0 {
            self.set(token, value)
        } else {
            if let Some(parent) = &self.parent {
                parent.borrow_mut().set_at(token, value, depth - 1)
            } else {
                LoxError::new_runtime(
                    token.line,
                    format!("Bad depth. Looking for depth {depth}, but no parent found."),
                )
            }
        }
    }
}
