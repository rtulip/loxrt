use crate::interpreter::Types;
#[derive(Debug)]
pub struct LoxErrorContainer {
    line: usize,
    message: String,
}

impl LoxErrorContainer {
    pub fn report(&self) {
        eprintln!("[line {}] Error: {}", self.line, self.message);
    }
}

pub enum LoxError {
    ScannerError(LoxErrorContainer),
    ParserErrors(Vec<LoxErrorContainer>),
    ResolutionError(LoxErrorContainer),
    RuntimeError(LoxErrorContainer),
    ReturnError(Types),
}

impl LoxError {
    pub fn new_scanner<T>(line: usize, message: String) -> Result<T, Self> {
        Err(LoxError::ScannerError(LoxErrorContainer { line, message }))
    }
    pub fn new_parser<T>(line: usize, message: String) -> Result<T, Self> {
        Err(LoxError::ParserErrors(vec![LoxErrorContainer {
            line,
            message,
        }]))
    }
    pub fn new_runtime<T>(line: usize, message: String) -> Result<T, Self> {
        Err(LoxError::RuntimeError(LoxErrorContainer { line, message }))
    }
    pub fn new_resolution<T>(line: usize, message: String) -> Result<T, Self> {
        Err(LoxError::ResolutionError(LoxErrorContainer {
            line,
            message,
        }))
    }
    pub fn new_return<T>(value: Types) -> Result<T, Self> {
        Err(LoxError::ReturnError(value))
    }

    fn code(&self) -> i32 {
        match self {
            LoxError::ScannerError(_) => 1,
            LoxError::ParserErrors(_) => 2,
            LoxError::RuntimeError(_) => 3,
            LoxError::ResolutionError(_) => 4,
            LoxError::ReturnError(_) => panic!("Shouldn't try to exit on a return error"),
        }
    }

    pub fn report(&self) {
        match self {
            LoxError::ScannerError(e)
            | LoxError::RuntimeError(e)
            | LoxError::ResolutionError(e) => e.report(),
            LoxError::ParserErrors(es) => {
                for e in es {
                    e.report()
                }
            }
            LoxError::ReturnError(_) => panic!("Shouldn't be reporting return errors."),
        }
    }

    pub fn exit(&self) {
        std::process::exit(self.code())
    }
}
