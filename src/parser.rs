use crate::ast::{Expr, Stmt};
use crate::error::{LoxError, LoxErrorCode};
use crate::tokens::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<LoxError>> {
        let mut stmts = vec![];
        let mut errors = vec![];
        while !self.is_at_end() {
            match self.declaration() {
                Err(mut e) => {
                    errors.append(&mut e);
                    self.syncronize();
                }
                Ok(stmt) => stmts.push(stmt),
            }
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(stmts)
        }
    }

    fn declaration(&mut self) -> Result<Stmt, Vec<LoxError>> {
        if self.matches(vec![TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, Vec<LoxError>> {
        let name = self.consume(
            TokenType::Identifier(String::new()),
            String::from("Expected variable name"),
        )?;

        let mut expr = None;
        if self.matches(vec![TokenType::Equal]) {
            expr = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            String::from("Expected `;` after variable declaration"),
        )?;
        Ok(Stmt::Var { name, expr })
    }

    fn statement(&mut self) -> Result<Stmt, Vec<LoxError>> {
        if self.matches(vec![TokenType::Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, Vec<LoxError>> {
        let expr = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            String::from("Expected `;` after value."),
        )?;
        Ok(Stmt::Print { expr })
    }

    fn expression_statement(&mut self) -> Result<Stmt, Vec<LoxError>> {
        let expr = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            String::from("Expected `;` after expression."),
        )?;
        Ok(Stmt::Expr { expr })
    }

    fn expression(&mut self) -> Result<Box<Expr>, Vec<LoxError>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Box<Expr>, Vec<LoxError>> {
        let expr = self.equality()?;
        if self.matches(vec![TokenType::Equal]) {
            let equals = self.previous();
            let assignment = self.assignment()?;

            if let Expr::Variable { name, .. } = *expr {
                return Ok(Box::new(Expr::Assignment {
                    name: name.clone(),
                    value: assignment,
                }));
            } else {
                return LoxError::new(
                    equals.line,
                    format!("Invalid assignment target: {}", expr.to_string()),
                    LoxErrorCode::ParserError,
                );
            }
        }

        return Ok(expr);
    }

    fn equality(&mut self) -> Result<Box<Expr>, Vec<LoxError>> {
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

    fn comparison(&mut self) -> Result<Box<Expr>, Vec<LoxError>> {
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

    fn term(&mut self) -> Result<Box<Expr>, Vec<LoxError>> {
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

    fn factor(&mut self) -> Result<Box<Expr>, Vec<LoxError>> {
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

    fn unary(&mut self) -> Result<Box<Expr>, Vec<LoxError>> {
        if self.matches(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Box::new(Expr::Unary { operator, right }))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Box<Expr>, Vec<LoxError>> {
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
        } else if self.matches(vec![TokenType::Identifier(String::new())]) {
            let name = self.previous();
            return Ok(Box::new(Expr::Variable { name }));
        } else {
            let tok = self.peek();
            LoxError::new(
                tok.line,
                format!("Unexpected Token: {}", tok),
                LoxErrorCode::ParserError,
            )
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

    fn consume(&mut self, typ: TokenType, message: String) -> Result<Token, Vec<LoxError>> {
        if self.check(typ) {
            Ok(self.advance())
        } else {
            LoxError::new(self.previous().line, message, LoxErrorCode::ParserError)
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

    fn syncronize(&mut self) {
        // Note: This was orignally included, but seems to cause bugs.
        //
        // ```lox
        // var foo // this was caught for missing `;`
        // var     // this didn't cause an error
        // ```
        //
        // self.advance();
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
