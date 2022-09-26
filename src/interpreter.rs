use crate::ast::{Expr, Stmt};
use crate::environment::Environment;
use crate::error::LoxError;
use crate::tokens::{Token, TokenType};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::SystemTime;

pub trait Callable {
    fn airity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Types>)
        -> Result<Types, LoxError>;
    fn to_string(&self) -> String;
}

impl<F> Callable for F
where
    F: Fn() -> Result<Types, LoxError>,
{
    fn airity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Types>,
    ) -> Result<Types, LoxError> {
        self()
    }

    fn to_string(&self) -> String {
        String::from("<native function>")
    }
}

struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Box<Stmt>>,
}

impl LoxFunction {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Box<Stmt>>) -> Types {
        Types::Callable(Rc::new(Box::new(LoxFunction { name, params, body })))
    }
}

impl Callable for LoxFunction {
    fn airity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        mut arguments: Vec<Types>,
    ) -> Result<Types, LoxError> {
        let env = Environment::new_child(&interpreter.global_env);
        arguments
            .drain(..)
            .enumerate()
            .for_each(|(i, arg)| env.borrow_mut().define(self.params[i].lexeme.clone(), arg));
        match interpreter.execute_block(&self.body, env) {
            Err(LoxError::ReturnError(typ)) => return Ok(typ),
            Err(e) => return Err(e),
            _ => Ok(Types::Nil),
        }
    }

    fn to_string(&self) -> String {
        format!("<fn {}>", self.name.lexeme)
    }
}

#[derive(Clone)]
pub enum Types {
    Number(f64),
    String(String),
    Bool(bool),
    Callable(Rc<Box<dyn Callable>>),
    Nil,
}

impl std::fmt::Debug for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Types::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Types::String(s) => f.debug_tuple("String").field(s).finish(),
            Types::Bool(b) => f.debug_tuple("Bool").field(b).finish(),
            Types::Nil => write!(f, "Nil"),
            Types::Callable(c) => write!(f, "{}", c.to_string()),
        }
    }
}

impl Types {
    pub fn is_truty(&self) -> bool {
        match self {
            Types::Nil => false,
            Types::Bool(b) => *b,
            _ => true,
        }
    }
    pub fn number(&self, token: &Token) -> Result<f64, LoxError> {
        match self {
            Types::Number(f) => Ok(*f),
            _ => LoxError::new_runtime(token.line, format!("Expected Number but found {self}")),
        }
    }

    pub fn bool(&self, token: &Token) -> Result<bool, LoxError> {
        match self {
            Types::Bool(b) => Ok(*b),
            _ => LoxError::new_runtime(token.line, format!("Expected Bool but found {self}")),
        }
    }

    pub fn string(&self, token: &Token) -> Result<String, LoxError> {
        match self {
            Types::String(s) => Ok(s.clone()),
            _ => LoxError::new_runtime(token.line, format!("Expected String but found {self}")),
        }
    }

    pub fn callable(&self, token: &Token) -> Result<Rc<Box<dyn Callable>>, LoxError> {
        match self {
            Types::Callable(c) => Ok(c.clone()),
            _ => LoxError::new_runtime(token.line, format!("Expected Callable but found {self}")),
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
            Types::Callable(c) => write!(f, "{}", c.to_string()),
        }
    }
}

pub struct Interpreter {
    pub global_env: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

fn clock() -> Result<Types, LoxError> {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => Ok(Types::Number(n.as_millis() as f64 / 1000.0)),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let environment = Environment::new();
        environment.borrow_mut().define(
            String::from("clock"),
            Types::Callable(Rc::new(Box::new(clock))),
        );
        Interpreter {
            global_env: environment.clone(),
            environment,
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Box<Stmt>>) -> Result<(), LoxError> {
        for stmt in statements {
            self.execute(&**stmt)?;
        }

        Ok(())
    }

    pub fn execute_block(
        &mut self,
        block: &Vec<Box<Stmt>>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), LoxError> {
        let prev = self.environment.clone();
        self.environment = environment;

        for stmt in block {
            match self.execute(&**stmt) {
                Err(e) => {
                    self.environment = prev;
                    return Err(e);
                }
                _ => (),
            }
        }

        self.environment = prev;
        Ok(())
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), LoxError> {
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
                    self.environment.borrow_mut().define(name.clone(), value);
                } else {
                    unreachable!()
                }
            }
            Stmt::Block { stmts } => {
                let prev = self.environment.clone();
                self.environment = Environment::new_child(&prev);
                for stmt in stmts {
                    if let Err(e) = self.execute(stmt) {
                        self.environment = prev;
                        return Err(e);
                    }
                }

                self.environment = prev;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaulate(condition)?.is_truty() {
                    self.execute(then_branch)?;
                } else if else_branch.is_some() {
                    let branch = else_branch.as_ref().unwrap();
                    self.execute(&**branch)?;
                }
            }
            Stmt::While { condition, body } => {
                while self.evaulate(condition)?.is_truty() {
                    self.execute(body)?;
                }
            }
            Stmt::Function { name, params, body } => {
                let func = LoxFunction::new(name.clone(), params.clone(), body.clone());
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), func);
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    let value = self.evaulate(value)?;
                    return LoxError::new_return(value);
                } else {
                    return LoxError::new_return(Types::Nil);
                }
            }
        };

        Ok(())
    }

    pub fn evaulate(&mut self, expression: &Box<Expr>) -> Result<Types, LoxError> {
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
                        _ => LoxError::new_runtime(
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
                        left.number(&operator)? > right.number(&operator)?
                    )),
                    TokenType::GreaterEqual => Ok(Types::Bool(
                        left.number(&operator)? >= right.number(&operator)?
                    )),
                    TokenType::Less => Ok(Types::Bool(
                        left.number(&operator)? < right.number(&operator)?
                    )),
                    TokenType::LessEqual => Ok(Types::Bool(
                        left.number(&operator)? <= right.number(&operator)?
                    )),
                    TokenType::EqualEqual => Ok(Types::Bool(right == left)),
                    TokenType::BangEqual => Ok(Types::Bool(right != left)),
                    _ => LoxError::new_runtime(operator.line, format!("Bad binary operator: {}", operator)),
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
                        _ => LoxError::new_runtime(
                            operator.line,
                            format!("Cannot perform Unary operator `-` on {right}"),
                        ),
                    },
                    TokenType::Bang => return Ok(Types::Bool(!right.is_truty())),
                    _ => LoxError::new_runtime(
                        operator.line,
                        format!("Bad Unary operator {:?}", operator.tok_typ),
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
                _ => LoxError::new_runtime(value.line, format!("Bad Token Literal: {value}")),
            },
            Expr::Variable { ref name } => Ok(self.environment.borrow().get(&name)?.clone()),
            Expr::Assignment {
                name: ref name_tok,
                ref value,
            } => {
                let result_val = self.evaulate(&value)?;
                self.environment
                    .borrow_mut()
                    .set(&name_tok, result_val.clone())?;
                Ok(result_val)
            }
            Expr::Logical {
                ref left,
                ref operator,
                ref right,
            } => {
                let left = self.evaulate(left)?;
                match operator.tok_typ {
                    TokenType::Or => {
                        if left.is_truty() {
                            Ok(left)
                        } else {
                            Ok(self.evaulate(right)?)
                        }
                    }
                    TokenType::And => {
                        if !left.is_truty() {
                            Ok(left)
                        } else {
                            Ok(self.evaulate(right)?)
                        }
                    }
                    _ => LoxError::new_runtime(operator.line, format!("Bad operator: {operator}")),
                }
            }
            Expr::Call {
                ref callee,
                ref arguments,
                ref paren,
            } => {
                let callee = self.evaulate(callee)?;
                let mut args = vec![];
                for arg in arguments {
                    args.push(self.evaulate(arg)?);
                }

                let function = callee.callable(paren)?;

                if function.airity() != args.len() {
                    return LoxError::new_runtime(
                        paren.line,
                        format!(
                            "Expected {} arguments, but got {}",
                            function.airity(),
                            args.len()
                        ),
                    );
                }
                Ok(function.call(self, args)?)
            }
        }
    }
}
