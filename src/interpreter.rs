use crate::ast::{Expr, Stmt};
use crate::environment::Environment;
use crate::error::{LoxError, LoxErrorCode};
use crate::tokens::{Token, TokenType};
use std::rc::Rc;

#[derive(Clone)]
pub enum Types {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl Types {
    pub fn is_truty(&self) -> bool {
        match self {
            Types::Nil => false,
            Types::Bool(b) => *b,
            _ => true,
        }
    }
    pub fn number(&self, token: &Token) -> Result<f64, Vec<LoxError>> {
        match self {
            Types::Number(f) => Ok(*f),
            _ => LoxError::new(
                token.line,
                format!("Expected Number but found {self}"),
                LoxErrorCode::InterpreterError,
            ),
        }
    }

    pub fn bool(&self, token: &Token) -> Result<bool, Vec<LoxError>> {
        match self {
            Types::Bool(b) => Ok(*b),
            _ => LoxError::new(
                token.line,
                format!("Expected Bool but found {self}"),
                LoxErrorCode::InterpreterError,
            ),
        }
    }

    pub fn string(&self, token: &Token) -> Result<String, Vec<LoxError>> {
        match self {
            Types::String(s) => Ok(s.clone()),
            _ => LoxError::new(
                token.line,
                format!("Expected String but found {self}"),
                LoxErrorCode::InterpreterError,
            ),
        }
    }
}

impl PartialEq for Types {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Types::Nil, Types::Nil) => true,
            (Types::String(s1), Types::String(s2)) => s1 == s2,
            (Types::Number(n1), Types::Number(n2)) => n1 == n2,
            (Types::Bool(b1), Types::Bool(b2)) => b1 == b2,
            _ => false,
        }
    }
}

impl std::fmt::Display for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Types::Number(n) => write!(f, "{n}"),
            Types::String(s) => write!(f, "{s}"),
            Types::Bool(b) => write!(f, "{b}"),
            Types::Nil => write!(f, "Nil"),
        }
    }
}

pub struct Interpreter {
    environment: Rc<Box<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<(), Vec<LoxError>> {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), Vec<LoxError>> {
        match stmt {
            Stmt::Expr { expr } => {
                self.evaulate(expr)?;
            }
            Stmt::Print { expr } => {
                let s = self.evaulate(expr)?;
                println!("{s}");
            }
            Stmt::Var { name, expr } => {
                let mut value = Types::Nil;
                if let Some(expr) = expr {
                    value = self.evaulate(expr)?;
                }

                if let TokenType::Identifier(name) = &name.tok_typ {
                    Rc::get_mut(&mut self.environment)
                        .expect("It should be safe to access the environment")
                        .define(name.clone(), value);
                } else {
                    unreachable!()
                }
            }
            Stmt::Block { stmts } => {
                let prev = self.environment.clone();
                self.environment = Environment::new_child(&prev);
                for stmt in stmts {
                    self.execute(stmt)?;
                }

                self.environment = prev;
            }
        }

        Ok(())
    }

    pub fn evaulate(&mut self, expression: &Box<Expr>) -> Result<Types, Vec<LoxError>> {
        match **expression {
            Expr::Binary {
                ref left,
                ref operator,
                ref right,
            } => {
                let left = self.evaulate(left)?;
                let right = self.evaulate(right)?;

                match operator.tok_typ {
                    TokenType::Minus => Ok(Types::Number(
                        left.number(&operator)? - right.number(&operator)?,
                    )),
                    TokenType::Plus => match (&left, &right) {
                        (Types::Number(left), Types::Number(right)) => {
                            Ok(Types::Number(left + right))
                        }
                        (Types::String(left), Types::String(right)) => {
                            Ok(Types::String(format!("{left}{right}")))
                        }
                        _ => LoxError::new(
                            operator.line,
                            format!("Invalid operands for operator `+`.\n\tCannot add `{left}` with `{right}`"),LoxErrorCode::InterpreterError,
                        ),
                    },
                    TokenType::Slash => Ok(Types::Number(
                        left.number(&operator)? / right.number(&operator)?,
                    )),
                    TokenType::Star => Ok(Types::Number(
                        left.number(&operator)? * right.number(&operator)?,
                    )),
                    TokenType::Greater => Ok(Types::Bool(
                        left.bool(&operator)? > right.bool(&operator)?
                    )),
                    TokenType::GreaterEqual => Ok(Types::Bool(
                        left.bool(&operator)? >= right.bool(&operator)?
                    )),
                    TokenType::Less => Ok(Types::Bool(
                        left.bool(&operator)? < right.bool(&operator)?
                    )),
                    TokenType::LessEqual => Ok(Types::Bool(
                        left.bool(&operator)? <= right.bool(&operator)?
                    )),
                    TokenType::EqualEqual => Ok(Types::Bool(right == left)),
                    TokenType::BangEqual => Ok(Types::Bool(right != left)),
                    _ => LoxError::new(operator.line, format!("Bad binary operator: {}", operator), LoxErrorCode::InterpreterError),
                }
            }
            Expr::Unary {
                ref operator,
                ref right,
            } => {
                let right = self.evaulate(right)?;
                match operator.tok_typ {
                    TokenType::Minus => match right {
                        Types::Number(n) => return Ok(Types::Number(-n)),
                        _ => LoxError::new(
                            operator.line,
                            format!("Cannot perform Unary operator `-` on {right}"),
                            LoxErrorCode::InterpreterError,
                        ),
                    },
                    TokenType::Bang => return Ok(Types::Bool(!right.is_truty())),
                    _ => LoxError::new(
                        operator.line,
                        format!("Bad Unary operator {:?}", operator.tok_typ),
                        LoxErrorCode::InterpreterError,
                    ),
                }
            }
            Expr::Grouping { ref expr } => self.evaulate(expr),
            Expr::Literal { ref value } => match &value.tok_typ {
                TokenType::Str(s) => Ok(Types::String(s.clone())),
                TokenType::Number(n) => Ok(Types::Number(*n)),
                TokenType::False => Ok(Types::Bool(false)),
                TokenType::True => Ok(Types::Bool(true)),
                TokenType::Nil => Ok(Types::Nil),
                _ => LoxError::new(
                    value.line,
                    format!("Bad Token Literal: {value}"),
                    LoxErrorCode::InterpreterError,
                ),
            },
            Expr::Variable { ref name } => Ok(self.environment.get(&name)?.clone()),
            Expr::Assignment {
                name: ref name_tok,
                ref value,
            } => {
                let result_val = self.evaulate(&value)?;
                Rc::get_mut(&mut self.environment)
                    .expect("It should be safe to access the environment")
                    .set(&name_tok, result_val.clone())?;
                Ok(result_val)
            }
        }
    }
}
