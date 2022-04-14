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

impl Compiler {
    fn create_node_from_tokens(&self,
                               starting_token: usize, 
                               available_functions: &BTreeMap<&str, &dyn CallableFunction>,
                               available_globals: &BTreeMap<&str, &dyn CallableGlobal>,
                               expected_type: ValueType) -> Result<Node, CompileError> {



        todo!()
    }

    fn create_function_parameters_for_node_from_tokens(&self,
                                                       node: &mut Node,
                                                       function_call_token: usize,
                                                       current_token: &mut usize,
                                                       available_functions: &BTreeMap<&str, &dyn CallableFunction>,
                                                       available_globals: &BTreeMap<&str, &dyn CallableGlobal>) -> Result<(), CompileError> {

        // Get the function
        let function_name = node.string_data.as_ref().unwrap().as_str();
        let function = match available_functions.get(function_name) {
            Some(n) => n,
            None => return_compile_error!(self, &self.tokens[function_call_token], &format!("function '{function_name}' is not defined"))
        };

        // Keep going until we have a )
        let mut parameter_index : usize = 0;
        loop {
            let token = &self.tokens[*current_token];

            // Done!
            if token.string == ")" {
                break;
            }

            // Get the next type. Or complain if this is impossible because we've hit the max number of parameters.
            let expected_type = match function.get_type_of_parameter(parameter_index) {
                Some(n) => n,
                None => return_compile_error!(self, token, &format!("function '{function_name}' takes at most {} parameter(s) but extraneous parameter(s) were given", function.get_total_parameter_count()))
            };

            parameter_index += 1;
        }

        // Did we get enough parameters?
        let minimum = function.get_minimum_parameter_count();

        if parameter_index < minimum {
            return_compile_error!(self, self.tokens[function_call_token], &format!("function '{function_name}' takes at least {minimum} parameter(s), got {parameter_index} instead"))
        }

        // Done!
        Ok(())
    }

    pub fn digest_tokens(&mut self) -> Result<(), CompileError> {
        let (mut scripts, mut globals) = {
            let mut scripts = Vec::<Script>::new();
            let mut globals = Vec::<Global>::new();
            
            let mut t = 0;

            loop {
                // Get the file/line/column
                let (file, line, column) = if let Some(n) = self.tokens.get_mut(t) {
                    // This should ALWAYS be true, and if not there's a bug with this library
                    debug_assert_eq!(n.string, "(");

                    // Good!
                    (n.file, n.line, n.column)
                }
                else {
                    // We're done
                    break;
                };

                // Increment t to get to the block type and step over the left parenthesis for the loop below this check
                t = t + 1;

                // Check the block type
                let object_type_token = &mut self.tokens[t];

                // Make it lowercase
                let object_type = &mut object_type_token.string;
                object_type.make_ascii_lowercase();

                // Add the thing
                match object_type as &str {
                    // Global
                    "global" => globals.push(Global {
                        value_type: {
                            let value_type_token = &mut self.tokens[t + 1];
                            let value_type_string = &mut value_type_token.string;
                            value_type_string.make_ascii_lowercase();

                            if let Some(n) = ValueType::from_str_underscore(value_type_string) {
                                n
                            }
                            else {
                                return_compile_error!(self, value_type_token, &format!("expected global value type, got {t} instead"));
                            }
                        },

                        name: {
                            let name_token = &mut self.tokens[t + 2];
                            let name_string = &mut name_token.string;

                            // The name of a global cannot be parenthesis
                            if name_string == ")" || name_string == "(" {
                                return_compile_error!(self, name_token, &format!("expected global name, got {name_string} instead"));
                            }

                            name_string.make_ascii_lowercase();
                            name_string.to_owned()
                        },

                        original_token_offset: t - 1,
                        original_body_offset: t + 3,

                        node: None // we're going to parse this later
                    }),

                    // Script
                    "script" => {
                        // Get the script type
                        let script_type = {
                            let script_type_token = &mut self.tokens[t + 1];
                            let script_type_string = &mut script_type_token.string;
                            script_type_string.make_ascii_lowercase();

                            if let Some(n) = ScriptType::from_str(script_type_string) {
                                n
                            }
                            else {
                                return_compile_error!(self, script_type_token, &format!("expected script type, got {t} instead"));
                            }
                        };

                        // Get the return value if applicable
                        let (return_type, name_offset) = if !script_type.always_returns_void() {
                            let value_type_token = &mut self.tokens[t + 1];
                            let value_type_string = &mut value_type_token.string;
                            value_type_string.make_ascii_lowercase();

                            if let Some(n) = ValueType::from_str_underscore(value_type_string) {
                                (n, t + 3)
                            }
                            else {
                                return_compile_error!(self, value_type_token, &format!("expected script value type, got {t} instead"));
                            }
                        }
                        else {
                            (ValueType::Void, t + 2)
                        };

                        // Get the name of the script
                        let script_name = {
                            let name_token = &mut self.tokens[name_offset];
                            let name_string = &mut name_token.string;

                            // The name of a script cannot be parenthesis
                            if name_string == ")" || name_string == "(" {
                                return_compile_error!(self, name_token, &format!("expected script name, got {name_string} instead"));
                            }

                            name_string.make_ascii_lowercase();

                            // Using 'begin' is not allowed
                            if name_string == "begin" {
                                return_compile_error!(self, name_token, &format!("function 'begin' cannot be overridden"));
                            }

                            name_string.to_owned()
                        };

                        // Add it
                        scripts.push(Script {
                            name: script_name,
                            return_type: return_type,
                            script_type: script_type,

                            original_token_offset: t - 1,
                            original_body_offset: name_offset + 1,

                            node: None // we're going to parse this later
                        })
                    },

                    t => return_compile_error!(self, object_type_token, &format!("expected 'global' or 'script', got {t} instead"))
                }

                // Find the end of this block
                let mut depth : usize = 1;
                while depth > 0 {
                    let this_token_str = self.tokens[t].string.as_str();

                    // Increment t
                    t = t + 1;

                    depth = match this_token_str {
                        "(" => depth + 1,
                        ")" => depth - 1,
                        _ => depth
                    };
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
            global_nodes.insert(g.get_name().to_owned(), self.create_node_from_tokens(g.original_body_offset, &callable_functions, &callable_globals, g.value_type)?);
        }

        // Now parse all the scripts
        for s in &scripts {
            let mut n = Node {
                value_type: s.return_type,
                node_type: NodeType::FunctionCall(true),
                string_data: Some("begin".to_owned()),
                data: NodeData::None,
                parameters: None
            };

            self.create_function_parameters_for_node_from_tokens(&mut n, s.original_body_offset, &mut s.original_body_offset.clone(), &callable_functions, &callable_globals)?;
            script_nodes.insert(s.get_name().to_owned(), n);
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
