use crate::ast::Expr;
use crate::tokens::{Token, TokenType};
pub struct Interpreter;

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
    pub fn number(&self, token: &Token) -> Result<f64, InterpreterError> {
        match self {
            Types::Number(f) => Ok(*f),
            _ => InterpreterError::new(token.line, format!("Expected Number but found {self}")),
        }
    }

    pub fn bool(&self, token: &Token) -> Result<bool, InterpreterError> {
        match self {
            Types::Bool(b) => Ok(*b),
            _ => InterpreterError::new(token.line, format!("Expected Bool but found {self}")),
        }
    }

    pub fn string(&self, token: &Token) -> Result<String, InterpreterError> {
        match self {
            Types::String(s) => Ok(s.clone()),
            _ => InterpreterError::new(token.line, format!("Expected String but found {self}")),
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
            Types::String(s) => write!(f, "\"{s}\""),
            Types::Bool(b) => write!(f, "{b}"),
            Types::Nil => write!(f, "Nil"),
        }
    }
}

pub struct InterpreterError {
    pub line: usize,
    pub message: String,
}

impl InterpreterError {
    pub fn new<T>(line: usize, message: String) -> Result<T, Self> {
        Err(InterpreterError { line, message })
    }
}

impl Interpreter {
    pub fn evaulate(&self, expression: Box<Expr>) -> Result<Types, InterpreterError> {
        match *expression {
            Expr::Binary {
                left,
                operator,
                right,
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
                        _ => InterpreterError::new(
                            operator.line,
                            format!("Invalid operands for operator `+`.\n\tCannot add `{left}` with `{right}`"),
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
                    _ => InterpreterError::new(operator.line, format!("Bad binary operator: {}", operator)),
                }
            }
            Expr::Grouping { expr } => self.evaulate(expr),
            Expr::Literal { value } => match value.tok_typ {
                TokenType::Str(s) => Ok(Types::String(s.clone())),
                TokenType::Number(n) => Ok(Types::Number(n)),
                TokenType::False => Ok(Types::Bool(false)),
                TokenType::True => Ok(Types::Bool(true)),
                TokenType::Nil => Ok(Types::Nil),
                _ => InterpreterError::new(value.line, format!("Bad Token Literal: {value}")),
            },
            Expr::Unary { operator, right } => {
                let right = self.evaulate(right)?;
                match operator.tok_typ {
                    TokenType::Minus => match right {
                        Types::Number(n) => return Ok(Types::Number(-n)),
                        _ => InterpreterError::new(
                            operator.line,
                            format!("Cannot perform Unary operator `-` on {right}"),
                        ),
                    },
                    TokenType::Bang => return Ok(Types::Bool(!right.is_truty())),
                    _ => InterpreterError::new(
                        operator.line,
                        format!("Bad Unary operator {:?}", operator.tok_typ),
                    ),
                }
            }
        }
    }
}
