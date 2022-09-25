use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier(String),
    Str(String),
    Number(f64),
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    EoF,
}

impl PartialEq for TokenType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TokenType::LeftParen, TokenType::LeftParen)
            | (TokenType::RightParen, TokenType::RightParen)
            | (TokenType::LeftBrace, TokenType::LeftBrace)
            | (TokenType::RightBrace, TokenType::RightBrace)
            | (TokenType::Comma, TokenType::Comma)
            | (TokenType::Dot, TokenType::Dot)
            | (TokenType::Minus, TokenType::Minus)
            | (TokenType::Plus, TokenType::Plus)
            | (TokenType::Semicolon, TokenType::Semicolon)
            | (TokenType::Slash, TokenType::Slash)
            | (TokenType::Star, TokenType::Star)
            | (TokenType::Bang, TokenType::Bang)
            | (TokenType::BangEqual, TokenType::BangEqual)
            | (TokenType::Equal, TokenType::Equal)
            | (TokenType::EqualEqual, TokenType::EqualEqual)
            | (TokenType::Greater, TokenType::Greater)
            | (TokenType::GreaterEqual, TokenType::GreaterEqual)
            | (TokenType::Less, TokenType::Less)
            | (TokenType::LessEqual, TokenType::LessEqual)
            | (TokenType::Identifier(_), TokenType::Identifier(_))
            | (TokenType::Str(_), TokenType::Str(_))
            | (TokenType::Number(_), TokenType::Number(_))
            | (TokenType::And, TokenType::And)
            | (TokenType::Class, TokenType::Class)
            | (TokenType::Else, TokenType::Else)
            | (TokenType::False, TokenType::False)
            | (TokenType::Fun, TokenType::Fun)
            | (TokenType::For, TokenType::For)
            | (TokenType::If, TokenType::If)
            | (TokenType::Nil, TokenType::Nil)
            | (TokenType::Or, TokenType::Or)
            | (TokenType::Print, TokenType::Print)
            | (TokenType::Return, TokenType::Return)
            | (TokenType::Super, TokenType::Super)
            | (TokenType::This, TokenType::This)
            | (TokenType::True, TokenType::True)
            | (TokenType::Var, TokenType::Var)
            | (TokenType::While, TokenType::While)
            | (TokenType::EoF, TokenType::EoF) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok_typ: TokenType,
    _lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(tok_typ: TokenType, lexeme: String, line: usize) -> Self {
        Token {
            tok_typ,
            _lexeme: lexeme,
            line,
        }
    }

    pub fn keywords() -> HashMap<&'static str, TokenType> {
        let mut map = HashMap::new();
        map.insert("and", TokenType::And);
        map.insert("class", TokenType::Class);
        map.insert("else", TokenType::Else);
        map.insert("false", TokenType::False);
        map.insert("fun", TokenType::Fun);
        map.insert("for", TokenType::For);
        map.insert("if", TokenType::If);
        map.insert("nil", TokenType::Nil);
        map.insert("or", TokenType::Or);
        map.insert("print", TokenType::Print);
        map.insert("return", TokenType::Return);
        map.insert("super", TokenType::Super);
        map.insert("this", TokenType::This);
        map.insert("true", TokenType::True);
        map.insert("var", TokenType::Var);
        map.insert("while", TokenType::While);
        map
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.tok_typ)
    }
}
