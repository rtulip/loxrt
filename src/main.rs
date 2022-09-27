pub mod ast;
pub mod environment;
pub mod error;
pub mod interpreter;
pub mod parser;
pub mod resolver;
pub mod scanner;
pub mod tokens;

use error::LoxError;
use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use scanner::Scanner;
use std::fs;

pub struct Lox;
impl Lox {
    pub fn new() -> Self {
        Lox
    }

    pub fn run_file(&self, path: &str) -> Result<(), LoxError> {
        let s =
            fs::read_to_string(path).expect(format!("Failed to read from file: {}", path).as_str());
        self.run(s)
    }

    fn run(&self, source: String) -> Result<(), LoxError> {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;

        let mut interpreter = Interpreter::new();

        {
            let mut resolver = Resolver::new(&mut interpreter);
            resolver.resolve(&statements)?;
        }

        interpreter.interpret(&statements)?;

        Ok(())
    }
}

fn main() {
    let lox = Lox::new();
    if let Err(e) = lox.run_file("sample.lox") {
        e.report();
        e.exit();
    }
}
