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
                    if !value_type.can_convert_to(expected_type) {
                        return_compile_error!(self, token, format!("global '{literal_lowercase}' is '{}' which cannot convert to '{}'", value_type.as_str(), expected_type.as_str()))
                    }

                    // Use the global name as the string data
                    literal = self.lowercase_token(token);

                    // Great!
                    true
                }

                // It's not? I guess it's a literal. We'll worry about parsing it later.
                else {
                    false
                };

                Node {
                    value_type: expected_type,
                    node_type: NodeType::Primitive(is_global),
                    string_data: Some(literal),
                    data: None,
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

        // Can we convert the function type?
        let function_return_type = function.get_return_type();
        if !function_return_type.can_convert_to(expected_type) {
            return_compile_error!(self, function_call_token, format!("function '{function_name}' returns '{}' which cannot convert to '{}'", function_return_type.as_str(), expected_type.as_str()))
        }

        // Do we have enough parameters?
        let parameter_count = tokens.len();
        let minimum = function.get_minimum_parameter_count();
        if tokens.len() < minimum {
            return_compile_error!(self, function_call_token, format!("function '{function_name}' takes at least {minimum} parameter(s), got {parameter_count} instead"))
        }

        // Do we passthrough only the last parameter?
        let last_is_passthrough = function.is_passthrough_last();

        // Are we doing passthrough number?
        let is_number_passthrough = function.is_number_passthrough();
        let mut passthrough_number_type : Option<ValueType> = None;

        // If we return real, then we already know the type
        if is_number_passthrough && function_return_type == ValueType::Real {
            passthrough_number_type = Some(expected_type)
        }

        // Go through each token
        for parameter_index in 0..parameter_count {
            let token = &tokens[parameter_index];

            // Get the next type. Or complain if this is impossible because we've hit the max number of parameters.
            let parameter_expected_type = match function.get_type_of_parameter(parameter_index) {
                Some(ValueType::Passthrough) => if last_is_passthrough { ValueType::Void } else { expected_type },
                Some(n) => n,
                None => return_compile_error!(self, token, format!("function '{function_name}' takes at most {} parameter(s) but extraneous parameter(s) were given", function.get_total_parameter_count()))
            };

            // Make a node
            let new_node = self.create_node_from_tokens(token, parameter_expected_type, available_functions, available_globals)?;

            // Get the type
            if is_number_passthrough && matches!(passthrough_number_type, None) {
                match new_node.node_type {
                    NodeType::Primitive(true) => passthrough_number_type = Some(available_globals.get(new_node.string_data.as_ref().unwrap().as_str()).unwrap().get_value_type()),
                    NodeType::Primitive(false) => (),
                    NodeType::FunctionCall(_) => passthrough_number_type = Some(available_functions.get(new_node.string_data.as_ref().unwrap().as_str()).unwrap().get_return_type())
                }
            }

            // Add the parameter
            parameters.push(new_node);
        }

        // If we do number passthrough, try converting types
        if let Some(number_type) = passthrough_number_type {
            for parameter_index in 0..parameter_count {
                let parameter_node = &mut parameters[parameter_index];

                // Verify
                let actual_type = match parameter_node.node_type {
                    NodeType::Primitive(true) => available_globals.get(parameter_node.string_data.as_ref().unwrap().as_str()).unwrap().get_value_type(),
                    NodeType::Primitive(false) => number_type,
                    NodeType::FunctionCall(_) => available_functions.get(parameter_node.string_data.as_ref().unwrap().as_str()).unwrap().get_return_type()
                };

                if !actual_type.can_convert_to(number_type) {
                    return_compile_error!(self, tokens[parameter_index], format!("parameter #{parameter_index} changes to '{}' due to number passthrough, but it resolves to '{}' which cannot convert to the expected type", number_type.as_str(), actual_type.as_str()))
                }

                parameter_node.value_type = number_type;
            }
        }

        // Parse the values now
        for parameter_index in 0..parameter_count {
            let parameter_node = &mut parameters[parameter_index];

            if matches!(parameter_node.node_type, NodeType::Primitive(false)) {
                let parameter_token = &tokens[parameter_index];
                let string_to_parse = if function.is_uppercase_allowed_for_parameter(parameter_index) {
                    tokens[parameter_index].string.clone()
                }
                else {
                    self.lowercase_token(&tokens[parameter_index])
                };

                let string_to_parse_str = string_to_parse.as_str();
                let clear_string_data;

                parameter_node.data = match parameter_node.value_type {
                    ValueType::Boolean => {
                        clear_string_data = true;
                        match string_to_parse_str {
                            "0" | "false" | "off" => Some(NodeData::Boolean(false)),
                            "1" | "true" | "on" => Some(NodeData::Boolean(false)),
                            _ => return_compile_error!(self, tokens[parameter_index], format!("cannot parse literal '{string_to_parse}' as a boolean (expected 0/1/false/true/off/on)"))
                        }
                    },

                    ValueType::Short => {
                        clear_string_data = true;
                        match string_to_parse_str.parse::<i16>() {
                            Ok(n) => Some(NodeData::Short(n)),
                            Err(_) => return_compile_error!(self, tokens[parameter_index], format!("cannot parse literal '{string_to_parse}' as a short"))
                        }
                    },

                    ValueType::Long => {
                        clear_string_data = true;
                        match string_to_parse_str.parse::<i32>() {
                            Ok(n) => Some(NodeData::Long(n)),
                            Err(_) => return_compile_error!(self, tokens[parameter_index], format!("cannot parse literal '{string_to_parse}' as a long"))
                        }
                    },

                    ValueType::Real => {
                        clear_string_data = true;
                        match string_to_parse_str.parse::<f32>() {
                            Ok(n) => Some(NodeData::Real(n)),
                            Err(_) => return_compile_error!(self, tokens[parameter_index], format!("cannot parse literal '{string_to_parse}' as a real"))
                        }
                    },

                    ValueType::GameDifficulty => {
                        clear_string_data = false;
                        match string_to_parse_str {
                            "easy" => Some(NodeData::Short(0)),
                            "normal" => Some(NodeData::Short(1)),
                            "hard" => Some(NodeData::Short(2)),
                            "impossible" => Some(NodeData::Short(3)),
                            _ => return_compile_error!(self, tokens[parameter_index], format!("cannot parse literal '{string_to_parse}' as a game_difficulty (expected easy/normal/hard/impossible)"))
                        }
                    },

                    ValueType::Team => {
                        clear_string_data = false;
                        match string_to_parse_str {
                            // "none" => Some(NodeData::Short(0)), // none is not supported for some reason
                            "player" => Some(NodeData::Short(1)),
                            "human" => Some(NodeData::Short(2)),
                            "covenant" => Some(NodeData::Short(3)),
                            "flood" => Some(NodeData::Short(4)),
                            "sentinel" => Some(NodeData::Short(5)),
                            "unused6" => Some(NodeData::Short(6)),
                            "unused7" => Some(NodeData::Short(7)),
                            "unused8" => Some(NodeData::Short(8)),
                            "unused9" => Some(NodeData::Short(9)),
                            _ => return_compile_error!(self, tokens[parameter_index], format!("cannot parse literal '{string_to_parse}' as a team (expected player/human/covenant/flood/sentinel)"))
                        }
                    },

                    ValueType::Script => {
                        clear_string_data = false;
                        match available_functions.get(string_to_parse_str) {
                            Some(n) => match n.is_engine_function() {
                                true => return_compile_error!(self, tokens[parameter_index], format!("no script '{string_to_parse_str}' defined (a function is defined by this name, but it cannot be used here)")),
                                false => ()
                            },
                            None => return_compile_error!(self, tokens[parameter_index], format!("no script '{string_to_parse_str}' defined"))
                        };
                        None
                    }

                    _ => {
                        clear_string_data = false;
                        None
                    }
                };

                parameter_node.string_data = if clear_string_data {
                    None
                }
                else {
                    Some(string_to_parse)
                }
            }
        }

        // Set it
        Ok(Node {
            value_type: expected_type,
            node_type: NodeType::FunctionCall(function.is_engine_function()),
            string_data: Some(function_name),
            data: None,
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

        for s in &mut scripts {
            optimize_begin(s.node.as_mut().unwrap())
        }

        // Remove stubbed scripts
        'remove_stubs_loop: loop {
            let script_count = scripts.len();
            for i in 0..script_count {
                if scripts[i].script_type == ScriptType::Stub {
                    for j in 0..script_count {
                        if j == i || scripts[i].name != scripts[j].name { // ignore self and scripts that don't have the same name as self
                            continue
                        }
                        
                        // Is the script a static script?
                        if scripts[j].script_type != ScriptType::Static {
                            return_compile_error!(self, scripts[i].original_token, format!("cannot replace stub script '{}' with non-static script", scripts[i].name))
                        }
                        
                        // Does the type match?
                        if scripts[j].return_type != scripts[i].return_type {
                            return_compile_error!(self, scripts[i].original_token, format!("cannot replace stub script '{}' that returns '{}' with static script which returns '{}'", scripts[i].return_type.as_str(), scripts[i].name, scripts[j].return_type.as_str()))
                        }

                        // Okay, we can remove it
                        scripts.remove(i);

                        // Done
                        continue 'remove_stubs_loop;
                    }
                }
            }
            break;
        }

        // Ensure there are no duplicate scripts or globals
        let final_script_count = scripts.len();
        let final_global_count = globals.len();

        for i in 0..final_script_count {
            let script_name = &scripts[i].name;
            for j in i+1..final_script_count {
                if script_name == &scripts[j].name {
                    return_compile_error!(self, scripts[i].original_token, format!("multiple scripts '{script_name}' defined"))
                }
            }
        }

        for i in 0..final_global_count {
            let global_name = &globals[i].name;
            for j in i+1..final_global_count {
                if global_name == &globals[j].name {
                    return_compile_error!(self, globals[i].original_token, format!("multiple globals '{global_name}' defined"))
                }
            }
        }

        // Do we exceed the maximum number of scripts?
        if final_script_count > i16::MAX as usize {
            return_compile_error!(self, scripts[i16::MAX as usize + 1].original_token, format!("maximum script limit of {} exceeded ({} / {})", i16::MAX, final_script_count, i16::MAX));
        }

        // Find the script indices
        let scripts_by_index = {
            let mut sbi = BTreeMap::<String, i16>::new();
            for i in 0..final_script_count as usize {
                sbi.insert(scripts[i].name.clone(), i as i16);
            }
            sbi
        };

        fn find_script_indices_for_node(node: &mut Node, scripts: &BTreeMap::<String, i16>) {
            match node.node_type {
                NodeType::Primitive(false) => {
                    if node.value_type == ValueType::Script {
                        node.data = Some(NodeData::Short(*scripts.get(node.string_data.as_ref().unwrap()).unwrap()));
                    }
                },
                NodeType::Primitive(true) => (),
                NodeType::FunctionCall(_) => {
                    for p in node.parameters.as_mut().unwrap() {
                        find_script_indices_for_node(p, &scripts);
                    }
                }
            }
        }

        for s in &mut scripts {
            find_script_indices_for_node(s.node.as_mut().unwrap(), &scripts_by_index);
        }

        todo!()
    }
}
