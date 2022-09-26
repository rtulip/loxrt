use crate::tokens::Token;

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
}

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
}

impl Expr {
    pub fn to_string(&self) -> String {
        match &self {
            Expr::Binary {
                left,
                operator,
                right,
            } => format!("({} {operator} {})", left.to_string(), right.to_string()),
            Expr::Unary { operator, right } => format!("({operator} {})", right.to_string()),
            Expr::Grouping { expr } => format!("(group {})", expr.to_string()),
            Expr::Literal { value } => format!("{value}"),
            Expr::Variable { name } => format!("{name}"),
            Expr::Assignment { name, value } => format!("{name} = {} ", value.to_string()),
        }
    }
}
