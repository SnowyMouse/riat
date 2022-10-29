use std::fmt;
use std::ffi::{CStr, CString};

/// Error type for CompileError.
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

/// Diagnostic message generated on warning or error.
#[derive(Debug, Clone)]
pub struct CompileError {
    message: CString,
    file: CString,
    error_type: CompileErrorType,
    line: usize,
    column: usize
}

impl CompileError {
    /// Create a `CompileError` from the given parameters.
    pub(crate) fn from_message(file: &str, line: usize, column: usize, error_type: CompileErrorType, message: &str) -> CompileError {
        CompileError {
            line, column, error_type,
            file: CString::new(file).unwrap(), message: CString::new(message).unwrap()
        }
    }

    /// Get the message of the error.
    pub fn get_message(&self) -> &str {
        self.message.to_str().unwrap()
    }

    /// Get the filename.
    pub fn get_file(&self) -> &str {
        self.file.to_str().unwrap()
    }

    /// Get the message of the error.
    pub fn get_message_cstr(&self) -> &CStr {
        &self.message
    }

    /// Get the filename.
    pub fn get_file_cstr(&self) -> &CStr {
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
        write!(f, "{}:{}:{}: {}: {}", self.file.to_str().unwrap(), self.line, self.column, self.error_type, self.message.to_str().unwrap())
    }
}
