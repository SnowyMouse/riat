use super::*;
use super::definitions::{ALL_GLOBALS, ALL_FUNCTIONS, EngineFunction, EngineGlobal};

use std::collections::BTreeMap;

fn all_functions_and_globals_for_target(target: CompileTarget) -> (Vec<&'static EngineFunction>, Vec<&'static EngineGlobal>) {
    let mut functions = Vec::new();
    let mut globals = Vec::new();

    for f in &ALL_FUNCTIONS {
        if f.supports_target(target) {
            functions.push(f);
        }
    }

    for g in &ALL_GLOBALS {
        if g.supports_target(target) {
            globals.push(g);
        }
    }

    return (functions, globals)
}

macro_rules! return_compile_error {
    ($compiler: expr, $token: expr, $message: expr) => {
        return Err(CompileError::from_message(&$compiler.files[$token.file], $token.line, $token.column, CompileErrorType::Error, $message))
    };
}

macro_rules! compile_warn {
    ($compiler: expr, $token: expr, $message: expr) => {
        $compiler.warnings.push(CompileError::from_message(&$compiler.files[$token.file], $token.line, $token.column, CompileErrorType::Warning, $message))
    };
}

impl Compiler {
    fn lowercase_token(&mut self, token: &Token) -> String {
        let mut t = token.string.clone();
        t.make_ascii_lowercase();

        if t != token.string {
            compile_warn!(self, token, format!("token '{}' contains uppercase characters and was made lowercase", token.string))
        }

        t
    }

    fn create_node_from_tokens(&mut self,
                               token: &Token, 
                               expected_type: ValueType,
                               available_functions: &BTreeMap<&str, &dyn CallableFunction>,
                               available_globals: &BTreeMap<&str, &dyn CallableGlobal>) -> Result<Node, CompileError> {
        let node = match token.children.as_ref() {
            Some(ref children) => {
                let function_name = self.lowercase_token(&children[0]);

                self.create_node_from_function(function_name, token, expected_type, &children[1..], available_functions, available_globals)?
            },
            None => {
                // Figure out if it's a global
                let mut literal = token.string.clone();
                let mut literal_lowercase = literal.clone();
                literal_lowercase.make_ascii_lowercase();

                let is_global = if let Some(global) = available_globals.get(literal_lowercase.as_str()) {
                    let value_type = global.get_value_type();
                    literal = self.lowercase_token(token);

                    // Great!
                    true
                }

                // It's not? I guess it's a literal. We'll worry about parsing it later.
                else {
                    false
                };

                Node {
                    value_type: ValueType::Unparsed,
                    node_type: NodeType::Primitive(is_global),
                    string_data: Some(literal),
                    data: NodeData::None,
                    parameters: None
                }
            }
        };

        Ok(node)
    }

    fn create_node_from_function(&mut self,
                                 function_name: String,
                                 function_call_token: &Token,
                                 expected_type: ValueType,
                                 tokens: &[Token],
                                 available_functions: &BTreeMap<&str, &dyn CallableFunction>,
                                 available_globals: &BTreeMap<&str, &dyn CallableGlobal>) -> Result<Node, CompileError> {
        let mut parameters = Vec::<Node>::new();

        // This should never be true. We will always have a type to convert to.
        debug_assert!(expected_type != ValueType::Passthrough);

        // Get the function
        let function = match available_functions.get(function_name.as_str()) {
            Some(n) => n,
            None => return_compile_error!(self, function_call_token, format!("function '{function_name}' is not defined"))
        };

        // Do we have enough parameters?
        let parameter_count = tokens.len();
        let minimum = function.get_minimum_parameter_count();
        if tokens.len() < minimum {
            return_compile_error!(self, function_call_token, format!("function '{function_name}' takes at least {minimum} parameter(s), got {parameter_count} instead"))
        }

        // Go through each token
        for parameter_index in 0..parameter_count {
            let token = &tokens[parameter_index];

            // Get the next type. Or complain if this is impossible because we've hit the max number of parameters.
            let mut parameter_expected_type = match function.get_type_of_parameter(parameter_index) {
                Some(n) => n,
                None => return_compile_error!(self, token, format!("function '{function_name}' takes at most {} parameter(s) but extraneous parameter(s) were given", function.get_total_parameter_count()))
            };

            // Make a node
            let new_node = self.create_node_from_tokens(token, parameter_expected_type, available_functions, available_globals)?;

            // Add the parameter
            parameters.push(new_node);
        }

        // Set it
        Ok(Node {
            value_type: ValueType::Unparsed,
            node_type: NodeType::FunctionCall(function.is_engine_function()),
            string_data: Some(function_name),
            data: NodeData::None,
            parameters: Some(parameters)
        })
    }

    pub fn digest_tokens(&mut self) -> Result<(), CompileError> {
        let (mut scripts, mut globals) = {
            let mut tokens : Vec<Token> = self.tokens.drain(..).collect();

            let mut scripts = Vec::<Script>::new();
            let mut globals = Vec::<Global>::new();

            for token in tokens {
                let children = token.children.as_ref().unwrap();

                // Get the object type
                let block_type = &children[0];
                match self.lowercase_token(block_type).as_str() {
                    "global" => {
                        // Make sure we have enough tokens here
                        match children.len() {
                            n if n < 4 => {
                                return_compile_error!(self, token, format!("incomplete global definition, expected (global <type> <name> <expression>)"));
                            },
                            n if n > 4 => {
                                return_compile_error!(self, children[4], format!("extraneous token in global definition (note: globals do not have implicit begin blocks)"));
                            },
                            4 => (),
                            _ => unreachable!()
                        }

                        // Add the global
                        globals.push(Global {
                            name: {
                                let global_name_token = &children[2];
                                if !matches!(global_name_token.children, None) {
                                    return_compile_error!(self, global_name_token, format!("expected global name, got a block instead"))
                                }
                                self.lowercase_token(&global_name_token)
                            },
                            value_type: {
                                let value_type_token = &children[1];
                                let value_type_string = self.lowercase_token(&value_type_token);
                                match ValueType::from_str_underscore(&value_type_string) {
                                    Some(ValueType::Passthrough) => return_compile_error!(self, value_type_token, format!("cannot define '{value_type_string}' globals")),
                                    Some(n) => n,
                                    None => return_compile_error!(self, value_type_token, format!("expected global value type, got '{value_type_string}' instead"))
                                }
                            },
                            original_token: token,
                            node: None // we're going to parse this later
                        });
                    },
                    "script" => {
                        // Get the script type
                        let script_type_token = match children.get(1) {
                            Some(n) => n,
                            None => return_compile_error!(self, token, format!("incomplete script definition, expected script type after 'script'"))
                        };
                        let script_type_string = self.lowercase_token(script_type_token);
                        let script_type = match ScriptType::from_str(&script_type_string) {
                            Some(n) => n,
                            None => return_compile_error!(self, script_type_token, format!("expected script type, got '{script_type_string}' instead"))
                        };
                        let type_expected = script_type.always_returns_void();

                        // Do we have enough tokens?
                        let minimum_number_of_tokens = script_type.expression_offset() + 1;
                        if children.len() < minimum_number_of_tokens {
                            return_compile_error!(self, token, format!("incomplete script definition, expected (script {script_type_string}{} <name> <expression(s)>)", if type_expected { "" } else { " <return type>" }))
                        }

                        // Add the script
                        scripts.push(Script {
                            name: {
                                let name_token = &children[minimum_number_of_tokens - 2];
                                if !matches!(name_token.children, None) {
                                    return_compile_error!(self, name_token, format!("expected script name, got a block instead (note: function parameters are not supported prior to Halo 3)"))
                                }

                                let name = self.lowercase_token(&name_token);
                                match name.as_str() {
                                    "begin" | "if" | "cond" => return_compile_error!(self, name_token, format!("function '{name}' cannot be overridden by a script")),
                                    _ => ()
                                }

                                name
                            },
                            return_type: if type_expected {
                                let return_type_token = &children[2];
                                let return_type_token_string = self.lowercase_token(return_type_token);

                                match ValueType::from_str_underscore(&return_type_token_string) {
                                    Some(ValueType::Passthrough) => return_compile_error!(self, return_type_token, format!("cannot define '{return_type_token_string}' scripts")),
                                    Some(n) => n,
                                    None => return_compile_error!(self, return_type_token, format!("expected global return value type, got '{return_type_token_string}' instead"))
                                }
                            }
                            else {
                                ValueType::Void
                            },
                            script_type: script_type,
                            original_token: token,

                            node: None // we're going to parse this later
                        });
                    },
                    n => return_compile_error!(self, block_type, format!("expected 'global' or 'script', got '{n}' instead"))
                }
            }
            
            (scripts, globals)
        };

        // Get all the things we can use
        let (callable_functions, callable_globals) = {
            let mut callable_functions = BTreeMap::<&str, &dyn CallableFunction>::new();
            let mut callable_globals = BTreeMap::<&str, &dyn CallableGlobal>::new();

            let (targeted_functions, targeted_globals) = all_functions_and_globals_for_target(self.target);

            // Add everything
            for f in targeted_functions {
                callable_functions.insert(f.get_name(), f);
            }
            for g in targeted_globals {
                callable_globals.insert(g.get_name(), g);
            }
            for s in &scripts {
                callable_functions.insert(s.get_name(), s);
            }
            for g in &globals {
                callable_globals.insert(g.get_name(), g);
            }

            // Done
            (callable_functions, callable_globals)
        };

        let mut global_nodes = BTreeMap::<String, Node>::new();
        let mut script_nodes = BTreeMap::<String, Node>::new();

        // Parse all the globals
        for g in &globals {
            global_nodes.insert(g.get_name().to_owned(), self.create_node_from_function("begin".to_owned(), &g.original_token, g.value_type, &g.original_token.children.as_ref().unwrap()[3..], &callable_functions, &callable_globals)?);
        }

        // Now parse all the scripts
        for s in &scripts {
            script_nodes.insert(s.get_name().to_owned(), self.create_node_from_function("begin".to_owned(), &s.original_token, s.return_type, &s.original_token.children.as_ref().unwrap()[s.script_type.expression_offset()..], &callable_functions, &callable_globals)?);
        }

        // Move all the globals and scripts
        for g in &mut globals {
            g.node = global_nodes.remove(g.get_name());
            debug_assert!(!matches!(g.node, None));
        }
        for s in &mut scripts {
            s.node = script_nodes.remove(s.get_name());
            debug_assert!(!matches!(s.node, None));
        }

        // Optimize 'begin' nodes with only one call
        fn optimize_begin(node_to_optimize: &mut Node) {
            while matches!(node_to_optimize.node_type, NodeType::FunctionCall(true)) && node_to_optimize.string_data.as_ref().unwrap() == "begin" {
                let mut parameters = node_to_optimize.parameters.as_mut().unwrap();
                if parameters.len() == 1 {
                    *node_to_optimize = parameters.pop().unwrap();
                }
                else {
                    break;
                }
            }

            // Optimize its parameters
            if let NodeType::FunctionCall(_) = node_to_optimize.node_type {
                for i in node_to_optimize.parameters.as_mut().unwrap() {
                    optimize_begin(i);
                }
            }
        }

        for g in &mut globals {
            optimize_begin(g.node.as_mut().unwrap())
        }

        todo!()
    }
}
