use crate::ast::{Expr, Stmt};
use crate::error::LoxError;
use crate::interpreter::{FunctionKind, Interpreter};
use crate::tokens::Token;
use std::collections::HashMap;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    function_kind: FunctionKind,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Resolver {
            interpreter,
            scopes: vec![],
            function_kind: FunctionKind::None,
        }
    }
    pub fn resolve(&mut self, statements: &Vec<Box<Stmt>>) -> Result<(), LoxError> {
        for stmt in statements {
            self.resolve_stmt(&*stmt)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), LoxError> {
        match stmt {
            Stmt::Function { name, params, body } => {
                self.declare(&name)?;
                self.define(&name);
                self.resolve_function(&params, &body, FunctionKind::Function)?;
            }
            Stmt::Expr { expr } => self.resolve_expr(&*expr)?,
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(&*condition)?;
                self.resolve_stmt(&*then_branch)?;
                if let Some(branch) = else_branch {
                    self.resolve_stmt(&*branch)?;
                }
            }
            Stmt::Print { expr } => self.resolve_expr(&*expr)?,
            Stmt::Return { keyword, value } => {
                if let FunctionKind::None = self.function_kind {
                    return LoxError::new_resolution(
                        keyword.line,
                        String::from("Can't return from top-level code."),
                    );
                }

                if let Some(value) = value {
                    self.resolve_expr(&*value)?;
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(&*condition)?;
                self.resolve_stmt(&*body)?;
            }
            Stmt::Var { name, expr } => {
                self.declare(name)?;
                if let Some(init) = expr {
                    self.resolve_expr(&*init)?;
                }
                self.define(name);
            }
            Stmt::Block { stmts } => {
                self.begin_scope();
                self.resolve(stmts)?;
                self.end_scope();
            }
            Stmt::Class { name, methods } => {
                self.declare(name)?;
                for method in methods {
                    match &**method {
                        Stmt::Function { params, body, .. } => {
                            self.resolve_function(params, body, FunctionKind::Method)?;
                        }
                        _ => unreachable!(),
                    }
                }
                self.define(name);
            }
        }

        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), LoxError> {
        match expr {
            Expr::Variable { name } => {
                if let Some(scope) = self.scopes.last() {
                    if let Some(init) = scope.get(&name.lexeme) {
                        if !init {
                            return LoxError::new_resolution(
                                name.line,
                                String::from("Can't read local var in it's own initializer"),
                            );
                        }
                    }
                }

                self.resolve_local(expr, name);
            }
            Expr::Assignment { name, value } => {
                self.resolve_expr(&*value)?;
                self.resolve_local(&*value, name);
            }
            Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => {
                self.resolve_expr(&*left)?;
                self.resolve_expr(&*right)?;
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                self.resolve_expr(&*callee)?;
                for arg in arguments {
                    self.resolve_expr(&*arg)?;
                }
            }
            Expr::Grouping { expr } => self.resolve_expr(&*expr)?,
            Expr::Literal { .. } => (),
            Expr::Unary { right, .. } => self.resolve_expr(&*right)?,
            Expr::Get { object, .. } => self.resolve_expr(&*object)?,
            Expr::Set { object, value, .. } => {
                self.resolve_expr(object)?;
                self.resolve_expr(value)?;
            }
        }
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<(), LoxError> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return LoxError::new_resolution(
                    name.line,
                    format!(
                        "A variable with name `{}` already exists within this scope",
                        name.lexeme
                    ),
                );
            }
            scope.insert(name.lexeme.clone(), false);
        }

        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, i);
            }
        }
    }

    fn resolve_function(
        &mut self,
        params: &Vec<Token>,
        body: &Vec<Box<Stmt>>,
        kind: FunctionKind,
    ) -> Result<(), LoxError> {
        let prev_kind = self.function_kind.clone();
        self.function_kind = kind;
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve(body)?;
        self.end_scope();

        self.function_kind = prev_kind;

        Ok(())
    }
}
