use crate::tokens::{Token, TokenType};
use crate::Lox;
use substring::Substring;

pub struct Scanner<'a> {
    lox: &'a mut Lox,
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(lox: &'a mut Lox, source: String) -> Self {
        Scanner {
            lox,
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TokenType::EoF, String::from(""), self.line));
        self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                if self.matches('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.matches('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.matches('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            '>' => {
                if self.matches('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '/' => {
                if self.matches('/') {
                    while self.peek(0) != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            ' ' | '\t' | '\r' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            c => {
                if c.is_ascii_digit() {
                    self.number()
                } else if c.is_alphabetic() {
                    self.identifier()
                } else {
                    self.lox
                        .error(self.line, format!("Unexpected character `{c}`"))
                }
            }
        }
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }

    fn add_token(&mut self, tok_typ: TokenType) {
        self.tokens.push(Token::new(
            tok_typ,
            String::from(self.source.substring(self.start, self.current)),
            self.line,
        ));
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&self, offset: usize) -> char {
        self.source
            .chars()
            .nth(self.current + offset)
            .unwrap_or('\0')
    }
    fn string(&mut self) {
        while self.peek(0) != '"' && !self.is_at_end() {
            if self.peek(0) == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.lox
                .error(self.line, String::from("Unterminated String."));
            return;
        }

        self.advance();

        self.add_token(TokenType::Str(String::from(
            self.source.substring(self.start + 1, self.current - 1),
        )));
    }

    fn number(&mut self) {
        while self.peek(0).is_ascii_digit() {
            self.advance();
        }
        if self.peek(0) == '.' && self.peek(1).is_ascii_digit() {
            self.advance();
            while self.peek(0).is_ascii_digit() {
                self.advance();
            }
        }

        if let Ok(n) = self
            .source
            .substring(self.start, self.current)
            .parse::<f64>()
        {
            self.add_token(TokenType::Number(n));
        } else {
            self.lox
                .error(self.line, String::from("Failed to parse number"));
        }
    }

    fn identifier(&mut self) {
        let keywords = Token::keywords();
        while self.peek(0).is_alphanumeric() || self.peek(0) == '_' {
            self.advance();
        }
        let ident = String::from(self.source.substring(self.start, self.current));

        if let Some(kw) = keywords.get(ident.as_str()) {
            self.add_token(kw.clone());
        } else {
            self.add_token(TokenType::Identifier(String::from(ident)));
        }
    }
}
