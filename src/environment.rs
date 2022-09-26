use crate::error::{LoxError, LoxErrorCode};
use crate::interpreter::Types;
use crate::tokens::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Types>,
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

    pub fn define(&mut self, name: String, value: Types) {
        self.values.insert(name, value);
    }

    pub fn get(&self, token: &Token) -> Result<Types, Vec<LoxError>> {
        if self.values.contains_key(&token.lexeme) {
            Ok(self.values.get(&token.lexeme).unwrap().clone())
        } else if self.parent.is_some() {
            self.parent.as_ref().unwrap().borrow().get(token)
        } else {
            LoxError::new(
                token.line,
                format!("Undefined variable `{}`.", token.lexeme),
                LoxErrorCode::InterpreterError,
            )
        }
    }

    pub fn set(&mut self, token: &Token, value: Types) -> Result<(), Vec<LoxError>> {
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
            LoxError::new(
                token.line,
                format!("Undefined variable: `{}`.", token.lexeme),
                LoxErrorCode::InterpreterError,
            )
        }
    }
}
