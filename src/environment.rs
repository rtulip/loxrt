use crate::error::{LoxError, LoxErrorCode};
use crate::interpreter::Types;
use crate::tokens::Token;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    parent: Option<Rc<Box<Environment>>>,
    values: HashMap<String, Types>,
}

impl Environment {
    pub fn new() -> Rc<Box<Self>> {
        Rc::new(Box::new(Environment {
            parent: None,
            values: HashMap::new(),
        }))
    }

    pub fn new_child(parent: &Rc<Box<Self>>) -> Rc<Box<Self>> {
        Rc::new(Box::new(Environment {
            parent: Some(parent.clone()),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: String, value: Types) {
        self.values.insert(name, value);
    }

    pub fn get(&self, token: &Token) -> Result<&Types, Vec<LoxError>> {
        if self.values.contains_key(&token.lexeme) {
            Ok(self.values.get(&token.lexeme).unwrap())
        } else if self.parent.is_some() {
            self.parent.as_ref().unwrap().get(token)
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
            if let Some(ref mut p) = self.parent {
                Ok(Rc::get_mut(p)
                    .expect("It should be safe to modify the parent environment")
                    .set(token, value)?)
            } else {
                unreachable!()
            }
        } else {
            LoxError::new(
                token.line,
                format!("Undefined variable: `{}`.", token.lexeme),
                LoxErrorCode::InterpreterError,
            )
        }
    }
}
