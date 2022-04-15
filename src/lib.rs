#![allow(dead_code, unused_variables, unused_mut)] // for now, don't warn about these

#[cfg(test)]
mod test; // test module for unit testing

mod definitions;
mod value_type;
mod error;
pub use error::{CompileErrorType, CompileError};

mod compile;
mod types;
pub use types::*;

mod token;
use token::Token;

pub use value_type::ValueType;

pub struct Compiler {
    tokens: Vec<Token>,
    files: Vec<String>,

    target: CompileTarget,
    warnings: Vec<CompileError>
}

/// Result of a successful compilation
pub struct CompiledScriptData {
    scripts: Vec<Script>,
    globals: Vec<Global>,
    files: Vec<String>,
    warnings: Vec<CompileError>
}

impl Compiler {
    pub fn new(target: CompileTarget) -> Compiler {
        Compiler { 
            tokens: Vec::new(),
            files: Vec::new(),

            target: target,
            warnings: Vec::new()
        }
    }

    /// Read the tokens from a u8 slice containing UTF-8 data.
    ///
    /// # Errors
    ///
    /// Errors if the script contains non-UTF8, if non-parenthesis tokens exist outside of a block, or if any parenthesis are unmatched.
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
