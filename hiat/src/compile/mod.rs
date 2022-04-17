use super::*;
use super::definitions::{ALL_GLOBALS, ALL_FUNCTIONS, EngineFunction, EngineGlobal};

use std::collections::BTreeMap;

use std::ffi::{CString, CStr};

mod types;
pub use self::types::*;

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
        return Err(CompileError::from_message($compiler, $compiler.files[$token.file].as_str(), $token.line, $token.column, CompileErrorType::Error, $message.as_str()))
    };
}

macro_rules! compile_warn {
    ($compiler: expr, $token: expr, $message: expr) => {
        $compiler.warnings.push(CompileError::from_message($compiler, $compiler.files[$token.file].as_str(), $token.line, $token.column, CompileErrorType::Warning, $message.as_str()))
    };
}

impl Compiler {
    /// Lowercase the token, warning if the result is different than the input.
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

                let final_type;

                let is_global = if let Some(global) = available_globals.get(literal_lowercase.as_str()) {
                    let value_type = global.get_value_type();

                    // Handle global type if passthrough
                    final_type = if expected_type == ValueType::Passthrough {
                        value_type
                    }
                    else {
                        if !value_type.can_convert_to(expected_type) {
                            return_compile_error!(self, token, format!("global '{literal_lowercase}' is '{}' which cannot convert to '{}'", value_type.as_str(), expected_type.as_str()))
                        }
                        expected_type
                    };

                    // Use the global name as the string data
                    literal = self.lowercase_token(token);

                    // Great!
                    true
                }

                // It's not? I guess it's a literal. We'll worry about parsing it later.
                else {
                    final_type = expected_type;
                    false
                };

                Node {
                    value_type: final_type,
                    node_type: NodeType::Primitive(is_global),
                    string_data: Some(literal),
                    data: None,
                    parameters: None,
                    index: None,

                    file: token.file,
                    line: token.line,
                    column: token.column
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
        
        // Special handling for the cond function, turning (cond (condition1 expression1...) (condition2 expression2...)) into (if condition1 (begin expression1...) (if condition2 (begin expression2...) ...)
        if function_name == "cond" {
            // Make sure we have somewhere first
            if tokens.is_empty() {
                return_compile_error!(self, function_call_token, format!("cond requires at least one set of expressions"))
            }

            // Make our if statements
            let mut if_tree = Vec::<Token>::new();
            for token in tokens {
                let fail = || {
                    return_compile_error!(self, token, format!("cond requires each parameter to be (<condition> <expression(s)>)"))
                };

                let children = match token.children.as_ref() {
                    None => return fail(),
                    Some(n) if n.len() < 2 => return fail(),
                    Some(n) => n
                };

                let condition = &children[0];
                let expressions = &children[1..];

                // Make the begin block (begin <expression(s)>)
                let mut expressions_vec = Vec::<Token>::new();
                expressions_vec.reserve(expressions.len() + 1); // +1 for begin
                expressions_vec.push(Token {
                    line: expressions[0].line,
                    column: expressions[0].column,
                    file: expressions[0].file,
                    string: "begin".to_owned(),
                    children: None
                });
                expressions_vec.extend_from_slice(expressions);
                let begin_block = Token {
                    line: expressions[0].line,
                    column: expressions[0].column,
                    file: expressions[0].file,
                    string: String::new(),
                    children: Some(expressions_vec)
                };

                // Make the if statement (if (condition) (begin whatever the heck))
                let mut if_expressions = Vec::<Token>::new();
                if_expressions.reserve(3 + 1); // +1 in case there's an else condition
                if_expressions.push(Token {
                    line: token.line,
                    column: token.column,
                    file: token.file,
                    string: "if".to_owned(),
                    children: None
                });
                if_expressions.push(condition.to_owned());
                if_expressions.push(begin_block);
                let if_block = Token {
                    line: token.line,
                    column: token.column,
                    file: token.file,
                    string: String::new(),
                    children: Some(if_expressions)
                };

                if_tree.push(if_block);
            }

            // Make them into things
            let tree_len = if_tree.len();
            for i in (0..tree_len-1).rev() { // go in reverse, appending n+1 to n's children n = 0
                let tail = if_tree.pop().unwrap(); // this will remove it from the end of the vector and do a move which should be pretty fast
                if_tree[i].children.as_mut().unwrap().push(tail);
            }
            debug_assert_eq!(if_tree.len(), 1); // we should have 1 left, right??

            // Now parse it
            return self.create_node_from_tokens(&if_tree.pop().unwrap(), expected_type, available_functions, available_globals);
        }

        // Get function information
        let function = match available_functions.get(function_name.as_str()) {
            Some(n) => n,
            None => return_compile_error!(self, function_call_token, format!("function '{function_name}' is not defined"))
        };
        let last_is_passthrough = function.is_passthrough_last();

        // Do we have enough parameters?
        let parameter_count = tokens.len();
        let minimum = function.get_minimum_parameter_count();
        if tokens.len() < minimum {
            return_compile_error!(self, function_call_token, format!("function '{function_name}' takes at least {minimum} parameter(s), got {parameter_count} instead"))
        }


        // If this function normally returns a passthrough, change it to expected_type (for now)
        let function_return_type = function.get_return_type();
        let final_type = if expected_type == ValueType::Passthrough {
            function_return_type
        }
        else {
            expected_type
        };

        // Determine the passthrough parameter type
        let mut passthrough_type : Option<ValueType> = {
            // If this is the "set" function, the passthrough type should be the global type.
            if function_name == "set" {
                let fn_token = &tokens[0];
                if !matches!(fn_token.children, None) {
                    return_compile_error!(self, function_call_token, "function 'set' cannot take a block as the variable name".to_owned())
                }
                let string_data = self.lowercase_token(fn_token);
                match available_globals.get(string_data.as_str()) {
                    Some(n) => Some(n.get_value_type()),
                    None => return_compile_error!(self, function_call_token, format!("parameter '{string_data}' is not a global variable name"))
                }
            }

            // Otherwise, if the function returns a passthrough but the function return type was replaced, then all passthrough types should be replaced with our new return type
            else if function_return_type == ValueType::Passthrough && final_type != function_return_type {
                Some(final_type)
            }

            // ...otherwise we'll figure it out when we do the parameters
            else {
                None
            }
        };


        // Go through each token and load them as parameters
        let mut parameters = Vec::<Node>::new();
        for parameter_index in 0..parameter_count {
            let token = &tokens[parameter_index];

            // Get the type of this parameter. Or complain if this is impossible because we've hit the max number of parameters.
            let parameter_is_passthrough;
            let parameter_expected_type = match function.get_type_of_parameter(parameter_index) {
                // Function takes a passthrough.
                Some(ValueType::Passthrough) => {
                    // If it's not the last parameter and we only passthrough the last parameter, do void for now.
                    if last_is_passthrough && parameter_index + 1 != parameter_count {
                        parameter_is_passthrough = false;
                        ValueType::Void
                    }

                    // Otherwise, it's a passthrough type, 
                    else {
                        match passthrough_type.as_ref() {
                            Some(n) => { parameter_is_passthrough = false; *n }
                            None => { parameter_is_passthrough = true; ValueType::Passthrough }
                        }
                    }
                },

                // Default
                Some(n) => { parameter_is_passthrough = false; n },

                // We exceeded the max number of parameters
                None => return_compile_error!(self, token, format!("function '{function_name}' takes at most {} parameter(s) but extraneous parameter(s) were given", function.get_total_parameter_count()))
            };


            // Make the node
            let new_node = self.create_node_from_tokens(token, parameter_expected_type, available_functions, available_globals)?;

            // Update passthrough if needed
            if parameter_is_passthrough && new_node.value_type != ValueType::Passthrough {
                passthrough_type = Some(new_node.value_type); 
            }

            // Add the parameter
            parameters.push(new_node);
        }


        // Set the index union to 0xFFFF for globals if set
        if function_name == "set" {
            debug_assert_eq!(parameters[0].node_type, NodeType::Primitive(true));
            parameters[0].index = Some(0xFFFF);
        }


        // If nothing was passed, treat everything passthrough as a real
        let final_passthrough_type = passthrough_type.unwrap_or(ValueType::Real);


        // If we do number passthrough, make sure our passthrough type is numeric
        if function.is_number_passthrough() && !final_passthrough_type.can_convert_to(ValueType::Real) {
            return_compile_error!(self, function_call_token, format!("passthrough parameters resolve to '{}', but function '{function_name}' takes only numeric parameters", final_passthrough_type.as_str()))
        }


        // Parse the literals now
        for parameter_index in 0..parameter_count {
            let parameter_node = &mut parameters[parameter_index];

            if matches!(parameter_node.node_type, NodeType::Primitive(false)) {
                let parameter_token = &tokens[parameter_index];
                let string_to_parse = if function.is_uppercase_allowed_for_parameter(parameter_index) {
                    parameter_token.string.clone()
                }
                else {
                    self.lowercase_token(parameter_token)
                };

                // Passthrough literals get converted into reals
                if parameter_node.value_type == ValueType::Passthrough {
                    parameter_node.value_type = final_passthrough_type;
                }

                // Begin parsing
                let string_to_parse_str = string_to_parse.as_str();
                let clear_string_data;

                parameter_node.data = match parameter_node.value_type {
                    ValueType::Boolean => {
                        clear_string_data = true;
                        match string_to_parse_str {
                            "0" | "false" | "off" => Some(NodeData::Boolean(false)),
                            "1" | "true" | "on" => Some(NodeData::Boolean(true)),
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
                    },

                    ValueType::Passthrough => unreachable!(),

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

        // Can we convert the function type?
        if expected_type != ValueType::Passthrough && !final_type.can_convert_to(expected_type) {
            return_compile_error!(self, function_call_token, format!("function '{function_name}' returns '{}' which cannot convert to '{}'", final_type.as_str(), expected_type.as_str()))
        }


        // Set it
        Ok(Node {
            value_type: final_type,
            node_type: NodeType::FunctionCall(function.is_engine_function()),
            string_data: Some(function_name),
            data: None,
            parameters: Some(parameters),
            index: None,

            file: function_call_token.file,
            line: function_call_token.line,
            column: function_call_token.column
        })
    }

    pub fn digest_tokens(&mut self) -> Result<CompiledScriptData, CompileError> {
        let (mut scripts, mut globals) = {
            let tokens : Vec<Token> = self.tokens.drain(..).collect();

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
                            node: Node::default() // we're going to parse this later
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
                        let type_expected = !script_type.always_returns_void();

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
                                    None => return_compile_error!(self, return_type_token, format!("expected script return value type, got '{return_type_token_string}' instead"))
                                }
                            }
                            else {
                                ValueType::Void
                            },
                            script_type: script_type,
                            original_token: token,

                            node: Node::default() // we're going to parse this later
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
            if g.name.len() > 31 {
                return_compile_error!(self, g.original_token, format!("global name '{}' exceeds 31 characters in length", g.name));
            }
            global_nodes.insert(g.get_name().to_owned(), self.create_node_from_function("begin".to_owned(), &g.original_token, g.value_type, &g.original_token.children.as_ref().unwrap()[3..], &callable_functions, &callable_globals)?);
        }

        // Now parse all the scripts
        for s in &scripts {
            if s.name.len() > 31 {
                return_compile_error!(self, s.original_token, format!("script name '{}' exceeds 31 characters in length", s.name));
            }
            script_nodes.insert(s.get_name().to_owned(), self.create_node_from_function("begin".to_owned(), &s.original_token, s.return_type, &s.original_token.children.as_ref().unwrap()[s.script_type.expression_offset()..], &callable_functions, &callable_globals)?);
        }

        // Move all the globals and scripts
        for g in &mut globals {
            g.node = global_nodes.remove(g.get_name()).unwrap();
        }
        for s in &mut scripts {
            s.node = script_nodes.remove(s.get_name()).unwrap();
        }

        // Optimize 'begin' nodes with only one call
        fn optimize_begin(node_to_optimize: &mut Node) {
            while matches!(node_to_optimize.node_type, NodeType::FunctionCall(true)) && node_to_optimize.string_data.as_ref().unwrap() == "begin" {
                let parameters = node_to_optimize.parameters.as_mut().unwrap();
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
            optimize_begin(&mut g.node)
        }

        for s in &mut scripts {
            optimize_begin(&mut s.node)
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

        // Find the script and global indices
        let scripts_by_index = {
            let mut sbi = BTreeMap::<String, i16>::new();
            for i in 0..final_script_count as usize {
                sbi.insert(scripts[i].name.clone(), i as i16);
            }
            sbi
        };
        let globals_by_index = {
            let mut gbi = BTreeMap::<String, i32>::new();
            for i in 0..final_global_count as usize {
                gbi.insert(globals[i].name.clone(), i as i32);
            }
            gbi
        };

        fn find_global_script_indices_for_node(node: &mut Node, scripts: &BTreeMap::<String, i16>, globals: &BTreeMap::<String, i32>) {
            match node.node_type {
                NodeType::Primitive(false) => {
                    if node.value_type == ValueType::Script {
                        node.data = Some(NodeData::Short(*scripts.get(node.string_data.as_ref().unwrap()).unwrap()));
                    }
                },
                NodeType::Primitive(true) => {
                    match globals.get(node.string_data.as_ref().unwrap()) {
                        Some(n) => node.data = Some(NodeData::Long(*n)),
                        None => ()
                    }
                },
                NodeType::FunctionCall(is_engine_function) => {
                    // If it's not an engine function, the node gets this index value.
                    if !is_engine_function {
                        let index = *scripts.get(node.string_data.as_ref().unwrap()).unwrap();
                        node.index = Some(index as u16);
                    }

                    for p in node.parameters.as_mut().unwrap() {
                        find_global_script_indices_for_node(p, scripts, globals);
                    }
                }
            }
        }
        for s in &mut scripts {
            find_global_script_indices_for_node(&mut s.node, &scripts_by_index, &globals_by_index);
        }

        // Detect uninitialized globals (and also find script indices)
        fn find_uninitialized_globals(node: &Node, globals: &[Global], compiler: &mut Compiler) {
            match node.node_type {
                NodeType::Primitive(true) => {
                    let global_name = node.string_data.as_ref().unwrap().as_str();
                    for g in globals {
                        if g.name == global_name {
                            compile_warn!(compiler, node, format!("use of uninitialized global '{}'", global_name));
                            break;
                        }
                    }
                },
                NodeType::FunctionCall(_) => for c in node.parameters.as_ref().unwrap() { find_uninitialized_globals(&c, globals, compiler); },
                _ => ()
            }
        }
        for i in 0..globals.len() {
            find_global_script_indices_for_node(&mut globals[i].node, &scripts_by_index, &globals_by_index);
            find_uninitialized_globals(&globals[i].node, &globals[i+1..], self);
        }

        // We should NOT have any passthrough stuff remaining
        #[cfg(debug_assertions)]
        {
            fn no_passthrough(node: &Node) {
                assert!(node.value_type != ValueType::Passthrough);
                assert!(node.value_type != ValueType::Unparsed);

                match node.parameters.as_ref() {
                    Some(n) => for i in n {
                        no_passthrough(i);
                    },
                    None => ()
                }
            }
            for s in &scripts {
                no_passthrough(&s.node);
            }
            for g in &globals {
                no_passthrough(&g.node);
            }
        }

        // All right, let's make our thing
        let mut compiled_scripts = Vec::new();
        let mut compiled_globals = Vec::new();
        let mut nodes = Vec::new();

        fn make_compiled_node_from_node(compiler: &Compiler, node: Node, node_array: &mut Vec<CompiledNode>) -> usize {
            // What type of node is it?
            match node.node_type {
                NodeType::Primitive(is_global) => {
                    // Globals need to have string data set
                    debug_assert!(!is_global || !matches!(node.string_data, None));

                    let result = node_array.len();
                    node_array.push(CompiledNode {
                        node_type: node.node_type,
                        value_type: node.value_type,
                        data: node.data,
                        string_data: match node.string_data { Some(n) => Some(compiler.encoding.encode_to_cstring(n.as_str())), None => None },
                        next_node: None,
                        index: node.index,

                        file: node.file,
                        column: node.column,
                        line: node.line
                    });
                    result
                },
                NodeType::FunctionCall(_) => {
                    let parameters = node.parameters.unwrap();

                    // First let's get this function call done and over with
                    let function_call_node = node_array.len();
                    let function_name_node = function_call_node + 1;
                    node_array.push(CompiledNode {
                        node_type: node.node_type,
                        value_type: node.value_type,
                        data: Some(NodeData::NodeOffset(function_name_node)),
                        string_data: None,
                        next_node: None,
                        index: node.index,

                        file: node.file,
                        column: node.column,
                        line: node.line
                    });

                    // Next get the function name out of the way
                    node_array.push(CompiledNode {
                        node_type: NodeType::Primitive(false),
                        value_type: ValueType::FunctionName,
                        data: Some(NodeData::Long(0)),
                        string_data: match node.string_data { Some(n) => Some(compiler.encoding.encode_to_cstring(n.as_str())), None => None },
                        next_node: None,
                        index: node.index,

                        file: node.file,
                        column: node.column,
                        line: node.line
                    });

                    // Let's get our parameters here now
                    let mut previous_node = function_name_node;
                    for p in parameters {
                        let next_node = make_compiled_node_from_node(compiler, p, node_array);
                        node_array[previous_node].next_node = Some(next_node);
                        previous_node = next_node;
                    }

                    // Done
                    function_call_node
                }
            }
        }

        for s in scripts {
            compiled_scripts.push(
                CompiledScript {
                    name: self.encoding.encode_to_cstring(s.name.as_str()),
                    value_type: s.return_type,
                    script_type: s.script_type,
                    first_node: make_compiled_node_from_node(self, s.node, &mut nodes),

                    file: s.original_token.file,
                    column: s.original_token.column,
                    line: s.original_token.line
                }
            )
        }
        for g in globals {
            compiled_globals.push(
                CompiledGlobal {
                    name: self.encoding.encode_to_cstring(g.name.as_str()),
                    value_type: g.value_type,
                    first_node: make_compiled_node_from_node(self, g.node, &mut nodes),

                    file: g.original_token.file,
                    column: g.original_token.column,
                    line: g.original_token.line
                }
            )
        }

        // Make the files
        let mut files = Vec::<CString>::new();
        for i in self.files.drain(..) {
            files.push(self.encoding.encode_to_cstring(i.as_str()));
        }

        // Done!
        Ok(CompiledScriptData {
            scripts: compiled_scripts,
            globals: compiled_globals,
            files: files,
            warnings: self.warnings.drain(..).collect(),
            nodes: nodes
        })
    }
}
