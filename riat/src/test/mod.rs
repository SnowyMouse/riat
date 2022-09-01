use super::*;

const HELLO_WORLD_HSC : &'static [u8] = include_bytes!("script/hello_world.hsc");

#[test]
fn test_tokenizer_hello_world() {
    let mut compiler = Compiler::new(CompileTarget::HaloCEA, CompileEncoding::Windows1252);
    compiler.read_script_data("hello_world.hsc", HELLO_WORLD_HSC).unwrap();

    assert_eq!(1, compiler.tokens.len()); // 1 script
    assert_eq!(5, compiler.tokens[0].children.as_ref().unwrap().len()); // 5 elements in the script - 'script', 'static', 'void', 'hello_world', and the body
    assert_eq!(2, compiler.tokens[0].children.as_ref().unwrap()[4].children.as_ref().unwrap().len()); // 2 elements in the body - 'print' and 'hello world'
}

#[test]
fn test_compiler_hello_world() {
    let mut compiler = Compiler::new(CompileTarget::HaloCEA, CompileEncoding::Windows1252);
    compiler.read_script_data("hello_world.hsc", HELLO_WORLD_HSC).unwrap();

    // Compile script data
    compiler.compile_script_data().unwrap();
}

#[test]
fn test_compatibility() {
    let test_compatibility_gbx_only_hsc = include_bytes!("script/test_compatibility_gbx_only.hsc");

    let mut compiler_cea = Compiler::new(CompileTarget::HaloCEA, CompileEncoding::Windows1252);
    compiler_cea.read_script_data("test_compatibility_gbx_only_hsc.hsc", test_compatibility_gbx_only_hsc).unwrap();
    assert!(matches!(compiler_cea.compile_script_data(), Err(_))); // this needs to fail

    let mut compiler_gbx = Compiler::new(CompileTarget::HaloCustomEdition, CompileEncoding::Windows1252);
    compiler_gbx.read_script_data("test_compatibility_gbx_only_hsc.hsc", test_compatibility_gbx_only_hsc).unwrap();
    assert!(matches!(compiler_gbx.compile_script_data(), Ok(_))); // this needs to pass
}

#[test]
fn test_compiler_script_parameters() {
    let test_script_parameters_hsc = include_bytes!("script/test_script_parameters.hsc");

    let mut compiler = Compiler::new(CompileTarget::HaloCEA, CompileEncoding::Windows1252);
    compiler.read_script_data("test_script_parameters.hsc", test_script_parameters_hsc).unwrap();

    let script_data = compiler.compile_script_data().unwrap();

    let globals = script_data.get_globals();
    assert_eq!(1, globals.len());
    let scripts = script_data.get_scripts();
    assert_eq!(1, scripts.len());
    let script = &scripts[0];

    let nodes = script_data.get_nodes();
    let script_node = &nodes[script.get_first_node_index()];
    assert_eq!(ValueType::Real, script_node.get_value_type()); // return type

    // Function call
    assert_eq!(NodeType::FunctionCall(true), script_node.get_type());

    // No other node after this
    assert_eq!(None, script_node.get_next_node_index());

    let division_function_name_node = match script_node.get_data() {
        Some(NodeData::NodeOffset(offset)) => &nodes[offset],
        _ => panic!()
    };

    // This should be a function name
    assert_eq!(ValueType::FunctionName, division_function_name_node.get_value_type());
    assert_eq!("/", division_function_name_node.get_string_data().unwrap());

    // Addition function call
    let addition_node = &nodes[division_function_name_node.get_next_node_index().unwrap()];
    assert_eq!(NodeType::FunctionCall(true), addition_node.get_type());
    let addition_function_name_node = match addition_node.get_data() {
        Some(NodeData::NodeOffset(offset)) => &nodes[offset],
        _ => panic!()
    };
    assert_eq!(ValueType::FunctionName, addition_function_name_node.get_value_type());
    assert_eq!("+", addition_function_name_node.get_string_data().unwrap());

    // These are local variables.
    let a_node = &nodes[addition_function_name_node.get_next_node_index().unwrap()];
    let b_node = &nodes[a_node.get_next_node_index().unwrap()];
    assert_eq!("a", a_node.get_string_data().unwrap());
    assert_eq!("b", b_node.get_string_data().unwrap());
    assert_eq!(NodeType::Primitive(PrimitiveType::Local), a_node.get_type());
    assert_eq!(NodeType::Primitive(PrimitiveType::Local), b_node.get_type());
    assert_eq!(NodeData::Long(0), a_node.get_data().unwrap());
    assert_eq!(NodeData::Long(1), b_node.get_data().unwrap());
    assert_eq!(None, b_node.get_next_node_index());

    // The "two" node we're dividing by.
    let two_node = &nodes[addition_node.get_next_node_index().unwrap()];
    assert_eq!("two", two_node.get_string_data().unwrap());
    assert_eq!(NodeType::Primitive(PrimitiveType::Global), two_node.get_type());
    assert_eq!(NodeData::Long(0), two_node.get_data().unwrap());
    assert_eq!(None, two_node.get_next_node_index());
}

#[test]
fn test_number_passthrough() {
    let mut compiler = Compiler::new(CompileTarget::HaloCEA, CompileEncoding::Windows1252);
    compiler.read_script_data("test_number_passthrough.hsc", include_bytes!("script/number_passthrough.hsc")).unwrap();

    // Compile script data
    let result = compiler.compile_script_data().unwrap();

    // Let's look at each global
    let globals = result.get_globals();
    let nodes = result.get_nodes();
    assert_eq!(3, globals.len());

    

    // Literal parameters passed to (+ <a> <b>) should be real
    let eleven_node = &nodes[globals[0].get_first_node_index()];
    assert_eq!(ValueType::Short, eleven_node.get_value_type()); // return type

    // Function name
    let eleven_node_function_name = match eleven_node.get_data() {
        Some(NodeData::NodeOffset(n)) => &nodes[n],
        _ => unreachable!()
    };
    assert_eq!(ValueType::FunctionName, eleven_node_function_name.get_value_type()); // function name
    assert_eq!("+", eleven_node_function_name.get_string_data().unwrap());

    // First parameter - 5
    let eleven_node_1st_parameter = &nodes[eleven_node_function_name.get_next_node_index().unwrap()];
    assert_eq!(ValueType::Real, eleven_node_1st_parameter.get_value_type());
    assert_eq!(Some(NodeData::Real(5.0)), eleven_node_1st_parameter.get_data());
    assert_eq!(None, eleven_node_1st_parameter.get_string_data());

    // Second parameter - 6
    let eleven_node_2nd_parameter = &nodes[eleven_node_1st_parameter.get_next_node_index().unwrap()];
    assert_eq!(ValueType::Real, eleven_node_2nd_parameter.get_value_type());
    assert_eq!(Some(NodeData::Real(6.0)), eleven_node_2nd_parameter.get_data());
    assert_eq!(None, eleven_node_2nd_parameter.get_string_data());

    // That's everything
    assert_eq!(None, eleven_node_2nd_parameter.get_next_node_index());

    

    // Globals passed to (- <a> <b>) should be real even if the globals, themselves, are short, as well as the return value
    let zero_node = &nodes[globals[1].get_first_node_index()];
    assert_eq!(zero_node.get_value_type(), ValueType::Short); // return type

    // Function name
    let zero_node_function_name = match zero_node.get_data() {
        Some(NodeData::NodeOffset(n)) => &nodes[n],
        _ => unreachable!()
    };
    assert_eq!(ValueType::FunctionName, zero_node_function_name.get_value_type()); // function name
    assert_eq!("-", zero_node_function_name.get_string_data().unwrap());

    // First parameter - 5
    let zero_node_1st_parameter = &nodes[zero_node_function_name.get_next_node_index().unwrap()];
    assert_eq!(ValueType::Real, zero_node_1st_parameter.get_value_type());
    assert_eq!(Some(NodeData::Long(0)), zero_node_1st_parameter.get_data());
    assert_eq!("eleven", zero_node_1st_parameter.get_string_data().unwrap());

    // Second parameter - 6
    let zero_node_2nd_parameter = &nodes[zero_node_1st_parameter.get_next_node_index().unwrap()];
    assert_eq!(ValueType::Real, zero_node_2nd_parameter.get_value_type());
    assert_eq!(Some(NodeData::Long(0)), zero_node_2nd_parameter.get_data());
    assert_eq!("eleven", zero_node_2nd_parameter.get_string_data().unwrap());

    // That's everything
    assert_eq!(None, zero_node_2nd_parameter.get_next_node_index());

    

    // But parameters passed to (= <a> <b>) should match the input type
    let eleven_is_greater_than_zero = &nodes[globals[2].get_first_node_index()];
    assert_eq!(eleven_is_greater_than_zero.get_value_type(), ValueType::Boolean); // return type

    // Function name
    let eleven_is_greater_than_zero_function_name = match eleven_is_greater_than_zero.get_data() {
        Some(NodeData::NodeOffset(n)) => &nodes[n],
        _ => unreachable!()
    };
    assert_eq!(ValueType::FunctionName, eleven_is_greater_than_zero_function_name.get_value_type()); // function name
    assert_eq!("=", eleven_is_greater_than_zero_function_name.get_string_data().unwrap());

    // First parameter - 5
    let eleven_is_greater_than_zero_1st_parameter = &nodes[eleven_is_greater_than_zero_function_name.get_next_node_index().unwrap()];
    assert_eq!(ValueType::Short, eleven_is_greater_than_zero_1st_parameter.get_value_type());
    assert_eq!(Some(NodeData::Long(0)), eleven_is_greater_than_zero_1st_parameter.get_data());
    assert_eq!("eleven", eleven_is_greater_than_zero_1st_parameter.get_string_data().unwrap());

    // Second parameter - 6
    let eleven_is_greater_than_zero_2nd_parameter = &nodes[eleven_is_greater_than_zero_1st_parameter.get_next_node_index().unwrap()];
    assert_eq!(ValueType::Short, eleven_is_greater_than_zero_2nd_parameter.get_value_type());
    assert_eq!(Some(NodeData::Long(1)), eleven_is_greater_than_zero_2nd_parameter.get_data());
    assert_eq!("zero", eleven_is_greater_than_zero_2nd_parameter.get_string_data().unwrap());

    // That's everything
    assert_eq!(None, eleven_is_greater_than_zero_2nd_parameter.get_next_node_index());
}
