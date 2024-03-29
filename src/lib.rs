#[cfg(test)]
mod test; // test module for unit testing

mod definitions;
mod value_type;
mod error;
pub use error::{CompileErrorType, CompileError};

mod compile;
pub use compile::*;

mod types;
pub use types::*;

mod token;
use token::Token;

pub use value_type::ValueType;

/// Compiler instance.
pub struct Compiler {
    tokens: Vec<Token>,
    files: Vec<String>,

    target: CompileTarget,
    warnings: Vec<CompileError>
}

impl Compiler {
    /// Instantiate a new compiler instance with the given compile target.
    pub fn new(target: CompileTarget) -> Compiler {
        Compiler {
            tokens: Vec::new(),
            files: Vec::new(),

            target: target,
            warnings: Vec::new()
        }
    }

    /// Read the tokens from a u8 slice containing string data.
    ///
    /// # Errors
    ///
    /// Errors if the script contains data that could not be decoded, if non-parenthesis tokens exist outside of a block, or if any parenthesis are unmatched.
    pub fn read_script_data(&mut self, filename: &str, script: &[u8]) -> Result<(), CompileError> {
        self.tokenize_script_data(filename, script)
    }

    /// Parse all loaded tokens and then clear the tokens if successful.
    ///
    /// # Errors
    ///
    /// Errors if the script data is invalid.
    pub fn compile_script_data(&mut self) -> Result<CompiledScriptData, CompileError> {
        self.digest_tokens()
    }
}
