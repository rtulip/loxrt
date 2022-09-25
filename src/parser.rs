use crate::ast::Expr;
use crate::tokens::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Box<Expr>, String> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Box<Expr>, String> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Box<Expr>, String> {
        let mut expr = self.comparison()?;
        while self.matches(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<Expr>, String> {
        let mut expr = self.term()?;

        while self.matches(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<Expr>, String> {
        let mut expr = self.factor()?;

        while self.matches(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<Expr>, String> {
        let mut expr = self.unary()?;

        while self.matches(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Box::new(Expr::Binary {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<Expr>, String> {
        if self.matches(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Box::new(Expr::Unary { operator, right }))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Box<Expr>, String> {
        if self.matches(vec![
            TokenType::False,
            TokenType::True,
            TokenType::Nil,
            TokenType::Number(0.0),
            TokenType::Str(String::from("")),
        ]) {
            return Ok(Box::new(Expr::Literal {
                value: self.previous(),
            }));
        }
        if self.matches(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                TokenType::RightParen,
                String::from("Expected `)` after expression"),
            )?;
            return Ok(Box::new(Expr::Grouping { expr }));
        } else {
            Err(format!("Unexpected Token: {}", self.peek()))
        }
    }

    fn matches(&mut self, types: Vec<TokenType>) -> bool {
        for typ in types {
            if self.check(typ) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, typ: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().tok_typ == typ
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn consume(&mut self, typ: TokenType, message: String) -> Result<Token, String> {
        if self.check(typ) {
            Ok(self.advance())
        } else {
            Err(message)
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.tokens[self.current].tok_typ, TokenType::EoF)
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn _syncronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if matches!(self.previous().tok_typ, TokenType::Semicolon) {
                return;
            }
            match self.peek().tok_typ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}
