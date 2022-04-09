use std::fmt;

/// Error type for CompileError
#[derive(Copy, Clone)]
pub enum CompileErrorType {
    /// Warning, typically for potentially bad, but not technically invalid code
    Warning,

    /// The code was invalid
    Error
}

/// Compile target to use
#[derive(Copy, Clone, PartialEq)]
pub enum CompileTarget {
    /// Halo: Combat Evolved Anniversary as released by 343 Industries on Windows.
    HaloCombatEvolvedAnniversary,

    /// Halo: Combat Evolved as released by Gearbox and MacSoft on Windows and Mac OS X, respectively.
    ///
    /// This also applies to the demo released by MacSoft.
    GearboxHaloCombatEvolved,

    /// Halo: Combat Evolved demo as released by Gearbox on Windows.
    ///
    /// This also applies to the un-updated CD version by Gearbox on Windows.
    ///
    /// This does not apply to the demo released by MacSoft for Mac OS X, as it's based on a newer version.
    GearboxHaloCombatEvolvedDemo,

    /// Halo Custom Edition as released by Gearbox on Windows.
    GearboxHaloCustomEdition,
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

pub struct CompileError {
    message: String,
    file: String,
    error_type: CompileErrorType,
    line: usize,
    column: usize
}

impl CompileError {
    /// Create a `CompileError` from the given parameters.
    pub fn from_message(file: &str, line: usize, column: usize, error_type: CompileErrorType, message: &str) -> CompileError {
        CompileError { line: line, column: column, error_type: error_type, file: file.to_owned(), message: message.to_owned() }
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
