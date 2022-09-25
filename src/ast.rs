use crate::tokens::Token;

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Literal {
        value: Token,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
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
            Expr::Grouping { expr } => format!("(group {})", expr.to_string()),
            Expr::Literal { value } => format!("{value}"),
            Expr::Unary { operator, right } => format!("({operator} {})", right.to_string()),
        }
    }
}