use std::fmt;

/// Error type for CompileError
#[derive(Copy, Clone, Debug)]
pub enum CompileErrorType {
    /// Warning, typically for potentially bad, but not technically invalid code
    Warning,

    /// The code was invalid
    Error
}

impl CompileErrorType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            CompileErrorType::Warning => "warning",
            CompileErrorType::Error => "error"
        }
    }
}

impl fmt::Display for CompileErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct CompileError {
    message: String,
    file: String,
    error_type: CompileErrorType,
    line: usize,
    column: usize
}

impl CompileError {
    /// Create a `CompileError` from the given parameters.
    pub(crate) fn from_message(file: &str, line: usize, column: usize, error_type: CompileErrorType, message: String) -> CompileError {
        CompileError { line: line, column: column, error_type: error_type, file: file.to_owned(), message: message }
    }

    /// Get the message of the error.
    pub fn get_message(&self) -> &str {
        &self.message
    }

    /// Get the filename.
    pub fn get_file(&self) -> &str {
        &self.file
    }

    /// Get the error type.
    pub fn get_error_type(&self) -> CompileErrorType {
        self.error_type
    }

    /// Return the line and column of the error token.
    pub fn get_position(&self) -> (usize, usize) {
        (self.line, self.column)
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}: {}: {}", self.file, self.line, self.column, self.error_type, self.message)
    }
}
