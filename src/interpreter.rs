use crate::ast::{Expr, Stmt};
use crate::environment::Environment;
use crate::error::LoxError;
use crate::tokens::{Token, TokenType};
use std::cell::RefCell;
use std::collections::HashMap;
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

#[derive(Debug, Clone)]
pub struct LoxFunction {
    name: Token,
    params: Vec<Token>,
    body: Vec<Box<Stmt>>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Stmt>>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Types {
        Types::Callable(LoxFunction {
            name,
            params,
            body,
            closure,
            is_initializer,
        })
    }

    pub fn bind(&self, instance: Types) -> Types {
        let env = Environment::new_child(&self.closure);
        env.borrow_mut().define(String::from("this"), instance);
        LoxFunction::new(
            self.name.clone(),
            self.params.clone(),
            self.body.clone(),
            env,
            self.is_initializer,
        )
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
        let env = Environment::new_child(&self.closure);
        arguments
            .drain(..)
            .enumerate()
            .for_each(|(i, arg)| env.borrow_mut().define(self.params[i].lexeme.clone(), arg));
        match interpreter.execute_block(&self.body, env) {
            Err(LoxError::ReturnError(typ)) => {
                if self.is_initializer && typ == Types::Nil {
                    Ok(self.closure.borrow().get_at(
                        &Token {
                            lexeme: String::from("this"),
                            line: 0,
                            tok_typ: TokenType::Identifier(String::from("this")),
                        },
                        0,
                    )?)
                } else {
                    Ok(typ)
                }
            }
            Err(e) => Err(e),
            _ => {
                if self.is_initializer {
                    Ok(self.closure.borrow().get_at(
                        &Token {
                            lexeme: String::from("this"),
                            line: 0,
                            tok_typ: TokenType::Identifier(String::from("this")),
                        },
                        0,
                    )?)
                } else {
                    Ok(Types::Nil)
                }
            }
        }
    }

    fn to_string(&self) -> String {
        format!("<fn {}>", self.name.lexeme)
    }
}

#[derive(Debug, Clone)]
pub struct LoxClassInstance {
    base: LoxClass,
    fields: HashMap<String, Types>,
}

impl LoxClassInstance {
    pub fn new(base: LoxClass) -> Self {
        LoxClassInstance {
            base,
            fields: HashMap::new(),
        }
    }

    pub fn get(this: &Rc<RefCell<Self>>, field: &Token) -> Result<Types, LoxError> {
        if this.borrow().fields.contains_key(&field.lexeme) {
            return Ok(this.borrow().fields.get(&field.lexeme).unwrap().clone());
        }
        if this.borrow().base.methods.contains_key(&field.lexeme) {
            if let Types::Callable(method) = this.borrow().base.methods.get(&field.lexeme).unwrap()
            {
                return Ok(method.bind(Types::ClassInstance(this.clone())));
            } else {
                unreachable!();
            }
        }

        LoxError::new_runtime(
            field.line,
            format!(
                "Instance of {} doesn't have a field `{}`",
                this.borrow().base.to_string(),
                field.lexeme
            ),
        )
    }

    pub fn get_mut(&mut self, field: &Token) -> Result<&mut Types, LoxError> {
        if self.fields.contains_key(&field.lexeme) {
            return Ok(self.fields.get_mut(&field.lexeme).unwrap());
        }
        if self.base.methods.contains_key(&field.lexeme) {
            return Ok(self.base.methods.get_mut(&field.lexeme).unwrap());
        }

        LoxError::new_runtime(
            field.line,
            format!(
                "Instance of {} doesn't have a field `{}`",
                self.base.to_string(),
                field.lexeme
            ),
        )
    }

    pub fn set_property(&mut self, field: &Token, value: Types) {
        self.fields.insert(field.lexeme.clone(), value);
    }
}

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Types>,
    superclass: Option<Box<LoxClass>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        methods: HashMap<String, Types>,
        superclass: Option<Box<LoxClass>>,
    ) -> Self {
        LoxClass {
            name,
            methods,
            superclass,
        }
    }

    fn new_instance(&self) -> Types {
        Types::ClassInstance(Rc::new(RefCell::new(LoxClassInstance::new(self.clone()))))
    }

    fn find_method(&self, method: &String) -> Option<Types> {
        if let Some(method) = self.methods.get(method) {
            return Some(method.clone());
        }

        if let Some(sc) = &self.superclass {
            sc.find_method(method)
        } else {
            None
        }
    }
}

impl Callable for LoxClass {
    fn airity(&self) -> usize {
        if let Some(Types::Callable(initializer)) = self.find_method(&String::from("init")) {
            initializer.airity()
        } else {
            0
        }
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Types>,
    ) -> Result<Types, LoxError> {
        let instance = self.new_instance();
        if let Some(Types::Callable(initializer)) = self.find_method(&String::from("init")) {
            if let Types::Callable(bound) = initializer.bind(instance) {
                bound.call(interpreter, arguments)
            } else {
                unreachable!()
            }
        } else {
            Ok(instance)
        }
    }

    fn to_string(&self) -> String {
        format!("<class {}>", self.name)
    }
}

#[derive(Clone)]
pub enum Types {
    Number(f64),
    String(String),
    Bool(bool),
    NativeFunc(Rc<Box<dyn Callable>>),
    Callable(LoxFunction),
    Class(LoxClass),
    ClassInstance(Rc<RefCell<LoxClassInstance>>),
    Nil,
}

impl std::fmt::Debug for Types {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Types::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Types::String(s) => f.debug_tuple("String").field(s).finish(),
            Types::Bool(b) => f.debug_tuple("Bool").field(b).finish(),
            Types::NativeFunc(func) => write!(f, "{}", func.to_string()),
            Types::Callable(c) => write!(f, "{}", c.to_string()),
            Types::Class(name) => f.debug_tuple("Class").field(name).finish(),
            Types::ClassInstance(instance) => {
                f.debug_tuple("ClassInstance").field(instance).finish()
            }
            Types::Nil => write!(f, "Nil"),
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
            Types::Callable(c) => {
                let trait_obj: Box<dyn Callable> = Box::new(c.clone());
                Ok(Rc::new(trait_obj))
            }
            Types::Class(c) => {
                let trait_obj: Box<dyn Callable> = Box::new(c.clone());
                Ok(Rc::new(trait_obj))
            }
            Types::NativeFunc(f) => Ok(f.clone()),
            _ => LoxError::new_runtime(token.line, format!("Expected Callable but found {self}")),
        }
    }

    pub fn instance(&self, token: &Token) -> Result<Rc<RefCell<LoxClassInstance>>, LoxError> {
        match self {
            Types::ClassInstance(instance) => Ok(instance.clone()),
            _ => LoxError::new_runtime(
                token.line,
                format!("Expected ClassInstance but found {self}"),
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
            Types::Class(class) => write!(f, "{}", class.to_string()),
            Types::ClassInstance(instance) => {
                write!(f, "instance of {}", instance.borrow().base.to_string())
            }
            Types::Callable(c) => write!(f, "{}", c.to_string()),
            Types::NativeFunc(func) => write!(f, "{}", func.to_string()),
            Types::Nil => write!(f, "Nil"),
        }
    }
}

pub struct Interpreter {
    pub global_env: Rc<RefCell<Environment>>,
    pub environment: Rc<RefCell<Environment>>,
    locals: HashMap<String, usize>,
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
            Types::NativeFunc(Rc::new(Box::new(clock))),
        );
        Interpreter {
            global_env: environment.clone(),
            environment,
            locals: HashMap::new(),
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
                let func = LoxFunction::new(
                    name.clone(),
                    params.clone(),
                    body.clone(),
                    self.environment.clone(),
                    false,
                );
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
            Stmt::Class {
                name,
                methods,
                superclass,
            } => {
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), Types::Nil);
                let superclass = match superclass {
                    None => None,
                    Some(superclass) => {
                        let sc = self.evaulate(superclass)?;
                        match &sc {
                            Types::Class(c) => {
                                let env = Environment::new_child(&self.environment);
                                env.borrow_mut().define(String::from("super"), sc.clone());
                                self.environment = env;
                                Some(Box::new(c.clone()))
                            }
                            _ => {
                                return LoxError::new_runtime(
                                    name.line,
                                    String::from("Superclass must be a class"),
                                )
                            }
                        }
                    }
                };

                let mut mtds: HashMap<String, Types> = HashMap::new();
                for method in methods {
                    match &**method {
                        Stmt::Function { name, params, body } => {
                            mtds.insert(
                                name.lexeme.clone(),
                                LoxFunction::new(
                                    name.clone(),
                                    params.clone(),
                                    body.clone(),
                                    self.environment.clone(),
                                    if name.lexeme == "init" { true } else { false },
                                ),
                            );
                        }
                        _ => unreachable!(),
                    }
                }

                let class =
                    Types::Class(LoxClass::new(name.lexeme.clone(), mtds, superclass.clone()));
                if superclass.is_some() {
                    let prev = self.environment.borrow().parent.as_ref().unwrap().clone();
                    self.environment = prev;
                }

                self.environment.borrow_mut().set(name, class)?;
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
            Expr::Variable { ref name } => Ok(self.lookup_variable(name, &*expression)?),
            Expr::Assignment {
                name: ref name_tok,
                ref value,
            } => {
                let result_val = self.evaulate(&value)?;
                match self.locals.get(&value.to_string()) {
                    Some(dist) => {
                        self.environment
                            .borrow_mut()
                            .set_at(name_tok, result_val.clone(), *dist)?
                    }
                    None => self
                        .global_env
                        .borrow_mut()
                        .set(name_tok, result_val.clone())?,
                }
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
            Expr::Get {
                ref object,
                ref name,
            } => {
                let obj = self.evaulate(object)?;
                match obj {
                    Types::ClassInstance(instance) => Ok(LoxClassInstance::get(&instance, name)?),
                    _ => LoxError::new_runtime(
                        name.line,
                        String::from("Only instance have properties."),
                    ),
                }
            }
            Expr::Set {
                ref object,
                ref value,
                ref name,
            } => match self.evaulate(object)? {
                Types::ClassInstance(instance) => {
                    instance
                        .borrow_mut()
                        .set_property(name, self.evaulate(value)?);
                    Ok(Types::Nil)
                }
                _ => todo!(),
            },
            Expr::This { ref keyword } => self.lookup_variable(&keyword, &expression),
            Expr::Super { ref method, .. } => {
                let dist = self.locals.get(&expression.to_string()).unwrap();
                let superclass = if let Types::Class(sc) = self.environment.borrow().get_at(
                    &Token {
                        lexeme: String::from("super"),
                        line: 0,
                        tok_typ: TokenType::Identifier(String::from("super")),
                    },
                    *dist,
                )? {
                    sc
                } else {
                    unreachable!()
                };

                let this = self.environment.borrow().get_at(
                    &Token {
                        lexeme: String::from("this"),
                        line: 0,
                        tok_typ: TokenType::Identifier(String::from("this")),
                    },
                    *dist - 1,
                )?;

                if let Some(Types::Callable(method)) = superclass.find_method(&method.lexeme) {
                    Ok(method.bind(this))
                } else {
                    LoxError::new_runtime(
                        method.line,
                        format!("Undefined property `{}`.", method.lexeme),
                    )
                }
            }
        }
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.to_string(), depth);
    }

    fn lookup_variable(&self, token: &Token, expr: &Expr) -> Result<Types, LoxError> {
        match self.locals.get(&expr.to_string()) {
            Some(dist) => self.environment.borrow().get_at(token, *dist),
            None => self.global_env.borrow().get(token),
        }
    }
}
