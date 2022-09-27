use crate::tokens::Token;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr {
        expr: Box<Expr>,
    },
    Print {
        expr: Box<Expr>,
    },
    Var {
        name: Token,
        expr: Option<Box<Expr>>,
    },
    Block {
        stmts: Vec<Box<Stmt>>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Box<Stmt>>,
    },
    Return {
        keyword: Token,
        value: Option<Box<Expr>>,
    },
    Class {
        name: Token,
        methods: Vec<Box<Stmt>>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: Token,
    },
    Variable {
        name: Token,
    },
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Box<Expr>>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
}

impl Expr {
    pub fn to_string(&self) -> String {
        match &self {
            Expr::Binary {
                left,
                operator,
                right,
            }
            | Expr::Logical {
                left,
                operator,
                right,
            } => format!("({} {operator} {})", left.to_string(), right.to_string()),
            Expr::Unary { operator, right } => format!("({operator} {})", right.to_string()),
            Expr::Grouping { expr } => format!("(group {})", expr.to_string()),
            Expr::Literal { value } => format!("{value}"),
            Expr::Variable { name } => format!("{name}"),
            Expr::Assignment { name, value } => format!("{name} = {} ", value.to_string()),
            Expr::Call {
                callee, arguments, ..
            } => {
                let mut s = format!("{}( ", callee.to_string());
                for arg in arguments {
                    s = format!("{s}{} ", arg.to_string());
                }
                s = format!("{s})");
                s
            }
            Expr::Get { object, .. } => format!("(get {})", object.to_string()),
            Expr::Set { object, value, .. } => {
                format!("(set {} <- {})", object.to_string(), value.to_string())
            }
        }
    }
}
