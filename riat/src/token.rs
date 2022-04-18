use super::*;

#[derive(Clone)]
pub(crate) struct Token {
    pub line: usize,
    pub column: usize,
    pub file: usize,
    pub string: String,

    pub children: Option<Vec<Token>>
}

impl Compiler {
    pub(super) fn tokenize_script_data(&mut self, filename: &str, script: &[u8]) -> Result<(), CompileError> {
        let mut tokens = Vec::<Token>::new();

        let file = self.files.len();
        let mut line : usize = 1;
        let mut column : usize = 0;

        // What are we currently in?
        let mut current_token_line : usize = 1;
        let mut current_token_column : usize = 1;
        let mut current_token_offset : usize = 0;

        enum CurrentlyIn {
            Whitespace,
            Token(bool), // if true, the token is terminated by a ". if false, the token is terminated by a whitespace or comment
            Comment(bool) // if true, the comment is terminated by a *; sequence. if false, the comment is terminated by a newline
        }

        let mut currently_in = CurrentlyIn::Whitespace;

        const ASTERISK : u8 = '*' as u8;

        let script_file_length = script.len();

        // Go through every character
        for i in 0..script_file_length {
            // Increment the column
            column = column + 1;

            let mut add_token = || {
                // Check if quoted
                let quoted = match currently_in {
                    CurrentlyIn::Token(quoted) => quoted,
                    _ => unreachable!("add_token() run on a non-token")
                };

                // Add it!
                tokens.push(Token {
                    line: current_token_line,
                    column: current_token_column,
                    file: file,
                    string: match self.encoding.decode_from_bytes(&script[current_token_offset + if quoted { 1 } else { 0 }..i]) {
                        Ok(n) => n.to_owned(),
                        Err(e) => return Err(CompileError::from_message(self, filename, line, column, CompileErrorType::Error, &format!("failed to decode token - {e}")))
                    },
                    children: None
                });

                // Done!
                Ok(())
            };

            // Get the character
            let c = script[i] as char;

            // Is it a null terminator?
            if c == 0 as char {
                if i + 1 == script_file_length {
                    break
                }
                else {
                    return Err(CompileError::from_message(self, filename, line, column, CompileErrorType::Error, "unexpected null terminator"))
                }
            }

            // If it's a special character, we take it
            if c == '(' || c == ')' {
                if matches!(currently_in, CurrentlyIn::Token(false)) {
                    add_token()?;
                    currently_in = CurrentlyIn::Whitespace;
                }

                if matches!(currently_in, CurrentlyIn::Whitespace) {
                    tokens.push(Token {
                        line: line,
                        column: column,
                        file: file,
                        string: c.to_string(),
                        children: None
                    });
                }
            }

            // If it's a whitespace, handle it
            else if c.is_whitespace() {
                // If it's non-quoted and we have a token, break it
                if matches!(currently_in, CurrentlyIn::Token(false)) {
                    add_token()?;
                    currently_in = CurrentlyIn::Whitespace;
                }

                // If it's a newline, advance the line by 1 and reset the column
                if c == '\n' {
                    line += 1;
                    column = 0;

                    // And if it's a single line comment, we're done
                    if matches!(currently_in, CurrentlyIn::Comment(false)) {
                        currently_in = CurrentlyIn::Whitespace;
                    }
                }
            }

            // Next, are we starting a comment?
            else if c == ';' {
                // Ending a token?
                if matches!(currently_in, CurrentlyIn::Token(false)) {
                    add_token()?;
                    currently_in = CurrentlyIn::Whitespace;
                }

                // Starting a comment?
                if matches!(currently_in, CurrentlyIn::Whitespace) {
                    currently_in = CurrentlyIn::Comment(matches!(&script.get(i + 1), Some(&ASTERISK))); // check if the next character is an asterisk. if so, it's terminated by a *;
                }

                // Ending a multi line comment?
                else if matches!(currently_in, CurrentlyIn::Comment(true)) && matches!(&script.get(i - 1), Some(&ASTERISK)) {
                    currently_in = CurrentlyIn::Whitespace;
                }
            }

            // Okay, are we starting a token?
            else if matches!(currently_in, CurrentlyIn::Whitespace) {
                currently_in = CurrentlyIn::Token(c == '"');
                current_token_line = line;
                current_token_column = column;
                current_token_offset = i;
            }

            // Are we ending a token?
            else if matches!(currently_in, CurrentlyIn::Token(true)) && c == '"' {
                add_token()?;
                currently_in = CurrentlyIn::Whitespace;
            }
        }

        // Did the token end prematurely?
        if let CurrentlyIn::Token(_) = currently_in {
            return Err(CompileError::from_message(self, filename, line, column, CompileErrorType::Error, "unterminated token"));
        }

        // Make the tokens into a tree
        let mut token_tree = Vec::<Token>::new();
        let mut token_iter = tokens.into_iter();

        loop {
            let mut next_token = match token_iter.next() {
                Some(n) => n,
                None => break
            };

            match next_token.string.as_str() {
                "(" => {
                    fn recursively_add_token(compiler: &Compiler, token: &mut Token, token_iter: &mut std::vec::IntoIter<Token>, filename: &str) -> Result<(), CompileError> {
                        let mut children = Vec::<Token>::new();
                        loop {
                            // Check if we have another token
                            let mut next_token = match token_iter.next() {
                                Some(n) => n,
                                None => return Err(CompileError::from_message(compiler, filename, token.line, token.column, CompileErrorType::Error, "unterminated block"))
                            };

                            // See if it's a parenthesis
                            match next_token.string.as_str() {
                                // It's another block!
                                "(" => {
                                    recursively_add_token(compiler, &mut next_token, token_iter, filename)?;
                                },

                                // We're closing the block
                                ")" => {
                                    // Error if a block is empty
                                    if children.is_empty() {
                                        return Err(CompileError::from_message(compiler, filename, token.line, token.column, CompileErrorType::Error, "empty block"))
                                    }

                                    // Move the token
                                    token.children = Some(children);

                                    // Done!
                                    return Ok(())
                                },

                                // Just an ordinary token with no children
                                _ => ()
                            }

                            // Okay, add it now
                            children.push(next_token);
                        }
                    }
                    recursively_add_token(self, &mut next_token, &mut token_iter, &filename)?;
                    token_tree.push(next_token)
                }

                n => {
                    return Err(CompileError::from_message(self, filename, line, column, CompileErrorType::Error, &format!("expected left parenthesis, got {n} instead")))
                }
            }
        }

        self.files.push(filename.to_owned());
        self.tokens.extend(token_tree);

        Ok(())
    }
}
