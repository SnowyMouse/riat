#![allow(dead_code, unused_variables)] // for now, don't warn about these

#[cfg(test)]
mod test;

mod definitions;
use definitions::CallableFunction;

mod value_type;

mod error;
pub use error::{CompileErrorType, CompileError};

mod compile;
pub use compile::{Script, Global};

mod token;
pub use token::Token;

pub use value_type::ValueType;

pub struct Compiler {
    tokens: Vec<Token>,
    files: Vec<String>
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler { 
            tokens: Vec::new(),
            files: Vec::new()
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

    /// Parse all loaded tokens into a script and then clear the tokens if successful.
    ///
    /// # Errors
    ///
    /// Errors if the script data is invalid.
    pub fn compile_script_data(&self) -> Result<(), CompileError> {
        self.digest_tokens()
    }
}
