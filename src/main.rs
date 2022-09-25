pub mod scanner;
pub mod tokens;

use scanner::Scanner;
use std::fs;

pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Lox { had_error: false }
    }

    pub fn run_file(&mut self, path: &str) {
        let s =
            fs::read_to_string(path).expect(format!("Failed to read from file: {}", path).as_str());
        self.run(s);
    }

    fn error(&mut self, line: usize, message: String) {
        self.report(line, String::from(""), message);
    }

    fn report(&mut self, line: usize, where_: String, message: String) {
        eprintln!("[line {line}] Error{where_}: {message}");
        self.had_error = true;
    }

    fn run(&mut self, source: String) {
        let scanner = Scanner::new(self, source);
        let tokens = scanner.scan_tokens();

        for tok in tokens {
            println!("{tok}");
        }
    }
}

fn main() {
    let mut lox = Lox::new();
    lox.run_file("sample.lox");
}
