#![allow(dead_code, unused_variables)] // for now, don't warn about these

#[cfg(test)]
mod test;
mod definitions;
mod value_type;

pub use value_type::ValueType;

pub struct Compiler {
    tokens: Vec<Token>,
    files: Vec<String>
}

impl Compiler {
    pub fn new() -> Self {
        Self { 
            tokens: Vec::new(),
            files: Vec::new()
        }
    }

    pub fn read_script_data(&mut self, filename: &str, script: &[u8]) -> Result<(), String> {
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

        // Go through every character
        for i in 0..script.len() {
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
                    string: match std::str::from_utf8(&script[current_token_offset + if quoted { 1 } else { 0 }..i]) {
                        Ok(n) => n.to_owned(),
                        Err(e) => return Err(format!("{filename}:{}:{}: error: failed to parse token - {e}", current_token_line, current_token_column))
                    }
                });

                // Done!
                Ok(())
            };

            // Increment the column
            column = column + 1;

            // Get the character
            let c = script[i] as char;

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
                        string: c.to_string()
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
            return Err(format!("{filename}:{}:{}: error: unterminated token", current_token_line, current_token_column));
        }

        // Make sure # of "(" = ")" and that anything else is in a block
        let mut depth : usize = 0;
        for i in &tokens {
            match i.string.as_str() {
                "(" => depth = depth + 1,
                ")" => depth = match depth.checked_sub(1) {
                    Some(n) => n,
                    None => return Err(format!("{filename}:{}:{}: error: unexpected right parenthesis", i.line, i.column))
                },
                n => if depth == 0 {
                    return Err(format!("{filename}:{}:{}: error: expected left parenthesis, got '{n}' instead", i.line, i.column))
                }
            }
        }

        self.files.push(filename.to_owned());
        self.tokens.extend(tokens);

        Ok(())
    }
}

struct Token {
    line: usize,
    column: usize,
    file: usize,
    string: String
}
