use crate::ast::{Expr, Stmt};
use crate::error::LoxError;
use crate::tokens::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Box<Stmt>>, LoxError> {
        let mut stmts = vec![];
        let mut errors = vec![];
        while !self.is_at_end() {
            match self.declaration() {
                Err(LoxError::ParserErrors(mut e)) => {
                    errors.append(&mut e);
                    self.syncronize();
                }
                Err(e) => {
                    return Err(e);
                }
                Ok(stmt) => stmts.push(stmt),
            }
        }

        if errors.len() > 0 {
            Err(LoxError::ParserErrors(errors))
        } else {
            Ok(stmts)
        }
    }

    fn declaration(&mut self) -> Result<Box<Stmt>, LoxError> {
        if self.matches(vec![TokenType::Var]) {
            return self.var_declaration();
        }
        if self.matches(vec![TokenType::Fun]) {
            return self.function("function");
        }
        self.statement()
    }

    fn function(&mut self, kind: &str) -> Result<Box<Stmt>, LoxError> {
        let name = self.consume(
            TokenType::Identifier(String::new()),
            format!("Expected {kind} name."),
        )?;

        self.consume(
            TokenType::LeftParen,
            format!("Expected `(` after {kind} name."),
        )?;

        let mut params = vec![];
        if !self.check(TokenType::RightParen) {
            while {
                if params.len() >= 255 {
                    return LoxError::new_parser(
                        self.peek().line,
                        String::from("Cannot have more than 255 parameters."),
                    );
                }

                params.push(self.consume(
                    TokenType::Identifier(String::new()),
                    format!("Expected parameter name. Found {}", self.peek()),
                )?);
                self.matches(vec![TokenType::Comma])
            } {}
        }

        self.consume(
            TokenType::RightParen,
            String::from("Expected `)` after parameters."),
        )?;

        self.consume(
            TokenType::LeftBrace,
            String::from("Expected `{` before {kind} body."),
        )?;

        let body = self.block()?;

        Ok(Box::new(Stmt::Function { name, params, body }))
    }

    fn var_declaration(&mut self) -> Result<Box<Stmt>, LoxError> {
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
        Ok(Box::new(Stmt::Var { name, expr }))
    }

    fn statement(&mut self) -> Result<Box<Stmt>, LoxError> {
        match self.advance().tok_typ {
            TokenType::Print => self.print_statement(),
            TokenType::Return => self.return_statement(),
            TokenType::LeftBrace => Ok(Box::new(Stmt::Block {
                stmts: self.block()?,
            })),
            TokenType::If => self.if_statement(),
            TokenType::While => self.while_statement(),
            TokenType::For => self.for_statement(),
            _ => {
                self.revert();
                self.expression_statement()
            }
        }
    }

    fn return_statement(&mut self) -> Result<Box<Stmt>, LoxError> {
        let keyword = self.previous();
        let mut value = None;
        if !self.check(TokenType::Semicolon) {
            value = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            String::from("Expect `;` after return value."),
        )?;

        Ok(Box::new(Stmt::Return { keyword, value }))
    }

    fn for_statement(&mut self) -> Result<Box<Stmt>, LoxError> {
        self.consume(
            TokenType::LeftParen,
            String::from("Expected `(` after `for`."),
        )?;
        let initializer = match self.advance().tok_typ {
            TokenType::Semicolon => None,
            TokenType::Var => Some(self.var_declaration()?),
            _ => {
                self.revert();
                Some(self.expression_statement()?)
            }
        };

        let condition = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(
            TokenType::Semicolon,
            String::from("Expect `;` after loop condition."),
        )?;

        let increment = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(
            TokenType::RightParen,
            String::from("Expect `)` after for clauses."),
        )?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Box::new(Stmt::Block {
                stmts: vec![body, Box::new(Stmt::Expr { expr: increment })],
            });
        }

        if let Some(condition) = condition {
            body = Box::new(Stmt::While { condition, body });
        }

        if let Some(initializer) = initializer {
            body = Box::new(Stmt::Block {
                stmts: vec![initializer, body],
            });
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Box<Stmt>, LoxError> {
        self.consume(
            TokenType::LeftParen,
            String::from("Expected `(` after `while`."),
        )?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            String::from("Expected `)` after condition"),
        )?;

        let body = self.statement()?;
        Ok(Box::new(Stmt::While { condition, body }))
    }

    fn if_statement(&mut self) -> Result<Box<Stmt>, LoxError> {
        self.consume(
            TokenType::LeftParen,
            String::from("Expected `(` after `if`."),
        )?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            String::from("Expected `)` after if condition."),
        )?;

        let then_branch = self.statement()?;
        let mut else_branch = None;
        if self.matches(vec![TokenType::Else]) {
            else_branch = Some(self.statement()?);
        }

        Ok(Box::new(Stmt::If {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn print_statement(&mut self) -> Result<Box<Stmt>, LoxError> {
        let expr = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            String::from("Expected `;` after value."),
        )?;
        Ok(Box::new(Stmt::Print { expr }))
    }

    fn block(&mut self) -> Result<Vec<Box<Stmt>>, LoxError> {
        let mut stmts = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(
            TokenType::RightBrace,
            String::from("Expected `}` at end of block."),
        )?;
        Ok(stmts)
    }

    fn expression_statement(&mut self) -> Result<Box<Stmt>, LoxError> {
        let expr = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            String::from("Expected `;` after expression."),
        )?;
        Ok(Box::new(Stmt::Expr { expr }))
    }

    fn expression(&mut self) -> Result<Box<Expr>, LoxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Box<Expr>, LoxError> {
        let expr = self.or()?;
        if self.matches(vec![TokenType::Equal]) {
            let equals = self.previous();
            let assignment = self.assignment()?;

            if let Expr::Variable { name, .. } = *expr {
                return Ok(Box::new(Expr::Assignment {
                    name: name.clone(),
                    value: assignment,
                }));
            } else {
                return LoxError::new_parser(
                    equals.line,
                    format!("Invalid assignment target: {}", expr.to_string()),
                );
            }
        }

        return Ok(expr);
    }

    fn or(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.and()?;

        while self.matches(vec![TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Box::new(Expr::Logical {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.equality()?;

        while self.matches(vec![TokenType::And]) {
            let operator = self.previous();
            let right = self.equality()?;

            expr = Box::new(Expr::Logical {
                left: expr,
                operator,
                right,
            });
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Box<Expr>, LoxError> {
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

    fn comparison(&mut self) -> Result<Box<Expr>, LoxError> {
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

    fn term(&mut self) -> Result<Box<Expr>, LoxError> {
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

    fn factor(&mut self) -> Result<Box<Expr>, LoxError> {
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

    fn unary(&mut self) -> Result<Box<Expr>, LoxError> {
        if self.matches(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Box::new(Expr::Unary { operator, right }))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Box<Expr>, LoxError> {
        let mut expr = self.primary()?;
        loop {
            if self.matches(vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Box<Expr>) -> Result<Box<Expr>, LoxError> {
        let mut arguments = vec![];
        if !self.check(TokenType::RightParen) {
            while {
                if arguments.len() >= 255 {
                    return LoxError::new_parser(
                        self.peek().line,
                        String::from("Cannnot have more than 255 arguments"),
                    );
                }
                arguments.push(self.expression()?);
                self.matches(vec![TokenType::Comma])
            } {}
        }

        let paren = self.consume(
            TokenType::RightParen,
            String::from("Expected `)` after arguments."),
        )?;

        Ok(Box::new(Expr::Call {
            callee,
            paren,
            arguments,
        }))
    }

    fn primary(&mut self) -> Result<Box<Expr>, LoxError> {
        let tok = self.advance();
        match &tok.tok_typ {
            TokenType::False
            | TokenType::True
            | TokenType::Nil
            | TokenType::Number(_)
            | TokenType::Str(_) => Ok(Box::new(Expr::Literal { value: tok })),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(
                    TokenType::RightParen,
                    String::from("Expected `)` after expression"),
                )?;
                Ok(Box::new(Expr::Grouping { expr }))
            }
            TokenType::Identifier(_) => Ok(Box::new(Expr::Variable { name: tok })),
            _ => LoxError::new_parser(tok.line, format!("Unexpected Token: {}", tok)),
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

    fn revert(&mut self) {
        self.current -= 1;
    }

    fn consume(&mut self, typ: TokenType, message: String) -> Result<Token, LoxError> {
        if self.check(typ) {
            Ok(self.advance())
        } else {
            LoxError::new_parser(self.previous().line, message)
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
