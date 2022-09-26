use crate::error::{LoxError, LoxErrorCode};
use crate::interpreter::Types;
use crate::tokens::Token;
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

    pub fn set(&mut self, token: &Token, value: Types) -> Result<(), Vec<LoxError>> {
        if self.globals.contains_key(&token.lexeme) {
            *self.globals.get_mut(&token.lexeme).unwrap() = value;
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
