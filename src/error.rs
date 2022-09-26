pub enum LoxErrorCode {
    ScannerError,
    InterpreterError,
    ParserError,
}

impl LoxErrorCode {
    fn code(self) -> i32 {
        match self {
            LoxErrorCode::ScannerError => 1,
            LoxErrorCode::InterpreterError => 2,
            LoxErrorCode::ParserError => 3,
        }
    }
}

pub struct LoxError {
    line: usize,
    message: String,
    code: LoxErrorCode,
}

impl LoxError {
    pub fn new<T>(line: usize, message: String, code: LoxErrorCode) -> Result<T, Self> {
        Err(LoxError {
            line,
            message,
            code,
        })
    }
    pub fn report(self) {
        eprintln!("[line {}] Error: {}", self.line, self.message);
        std::process::exit(self.code.code())
    }
}
