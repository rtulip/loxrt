pub mod ast;
pub mod environment;
pub mod error;
pub mod interpreter;
pub mod parser;
pub mod scanner;
pub mod tokens;

use error::LoxError;
use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use std::fs;

pub struct Lox;
impl Lox {
    pub fn new() -> Self {
        Lox
    }

    pub fn run_file(&self, path: &str) -> Result<(), Vec<LoxError>> {
        let s =
            fs::read_to_string(path).expect(format!("Failed to read from file: {}", path).as_str());
        self.run(s)
    }

    fn run(&self, source: String) -> Result<(), Vec<LoxError>> {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&statements)?;

        Ok(())
    }
}

fn main() {
    let lox = Lox::new();
    if let Err(errors) = lox.run_file("sample.lox") {
        for e in &errors {
            e.report();
        }

        if let Some(e) = errors.last() {
            e.exit();
        }
    }
}
