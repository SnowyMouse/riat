use super::*;

const HELLO_WORLD_HSC : &'static [u8] = include_bytes!("script/hello_world.hsc");

#[test]
fn test_tokenizer_hello_world() {
    let mut compiler = Compiler::new(CompileTarget::HaloCEA);
    compiler.read_script_data("hello_world.hsc", HELLO_WORLD_HSC).unwrap();

    assert_eq!(compiler.tokens.len(), 1); // 1 script
    assert_eq!(compiler.tokens[0].children.as_ref().unwrap().len(), 5); // 5 elements in the script - 'script', 'static', 'void', 'hello_world', and the body
    assert_eq!(compiler.tokens[0].children.as_ref().unwrap()[4].children.as_ref().unwrap().len(), 2); // 2 elements in the body - 'print' and 'hello world'
}

#[test]
fn test_compiler_hello_world() {
    let mut compiler = Compiler::new(CompileTarget::HaloCEA);
    compiler.read_script_data("hello_world.hsc", HELLO_WORLD_HSC).unwrap();

    // Compile script data
    compiler.compile_script_data().unwrap();
}

#[test]
fn test_compatibility() {
    let test_compatibility_gbx_only_hsc = include_bytes!("script/test_compatibility_gbx_only.hsc");

    let mut compiler_cea = Compiler::new(CompileTarget::HaloCEA);
    compiler_cea.read_script_data("test_compatibility_gbx_only_hsc.hsc", test_compatibility_gbx_only_hsc).unwrap();
    assert!(matches!(compiler_cea.compile_script_data(), Err(_))); // this needs to fail

    let mut compiler_gbx = Compiler::new(CompileTarget::HaloCustomEdition);
    compiler_gbx.read_script_data("test_compatibility_gbx_only_hsc.hsc", test_compatibility_gbx_only_hsc).unwrap();
    assert!(matches!(compiler_gbx.compile_script_data(), Ok(_))); // this needs to pass
}

#[test]
fn test_number_passthrough() {
    let mut compiler = Compiler::new(CompileTarget::HaloCEA);
    compiler.read_script_data("test_number_passthrough.hsc", include_bytes!("script/number_passthrough.hsc")).unwrap();

    // Compile script data
    let result = compiler.compile_script_data().unwrap();

    // Let's look at each global
    let globals = result.get_globals();
    let nodes = result.get_nodes();
    assert_eq!(globals.len(), 3);

    

    // Literal parameters passed to (+ <a> <b>) should be real
    let eleven_node = &nodes[globals[0].get_first_node_index()];
    assert_eq!(eleven_node.get_value_type(), ValueType::Short); // return type

    // Function name
    let eleven_node_function_name = match eleven_node.get_data() {
        Some(NodeData::NodeOffset(n)) => &nodes[n],
        _ => unreachable!()
    };
    assert_eq!(eleven_node_function_name.get_value_type(), ValueType::FunctionName); // function name
    assert_eq!(eleven_node_function_name.get_string_data().unwrap(), "+");

    // First parameter - 5
    let eleven_node_1st_parameter = &nodes[eleven_node_function_name.get_next_node_index().unwrap()];
    assert_eq!(eleven_node_1st_parameter.get_value_type(), ValueType::Real);
    assert_eq!(eleven_node_1st_parameter.get_data(), Some(NodeData::Real(5.0)));
    assert_eq!(eleven_node_1st_parameter.get_string_data(), None);

    // Second parameter - 6
    let eleven_node_2nd_parameter = &nodes[eleven_node_1st_parameter.get_next_node_index().unwrap()];
    assert_eq!(eleven_node_2nd_parameter.get_value_type(), ValueType::Real);
    assert_eq!(eleven_node_2nd_parameter.get_data(), Some(NodeData::Real(6.0)));
    assert_eq!(eleven_node_2nd_parameter.get_string_data(), None);

    // That's everything
    assert_eq!(eleven_node_2nd_parameter.get_next_node_index(), None);

    

    // Globals passed to (- <a> <b>) should be real even if the globals, themselves, are short, as well as the return value
    let zero_node = &nodes[globals[1].get_first_node_index()];
    assert_eq!(zero_node.get_value_type(), ValueType::Short); // return type

    // Function name
    let zero_node_function_name = match zero_node.get_data() {
        Some(NodeData::NodeOffset(n)) => &nodes[n],
        _ => unreachable!()
    };
    assert_eq!(zero_node_function_name.get_value_type(), ValueType::FunctionName); // function name
    assert_eq!(zero_node_function_name.get_string_data().unwrap(), "-");

    // First parameter - 5
    let zero_node_1st_parameter = &nodes[zero_node_function_name.get_next_node_index().unwrap()];
    assert_eq!(zero_node_1st_parameter.get_value_type(), ValueType::Real);
    assert_eq!(zero_node_1st_parameter.get_data(), None);
    assert_eq!(zero_node_1st_parameter.get_string_data().unwrap(), "eleven");

    // Second parameter - 6
    let zero_node_2nd_parameter = &nodes[zero_node_1st_parameter.get_next_node_index().unwrap()];
    assert_eq!(zero_node_2nd_parameter.get_value_type(), ValueType::Real);
    assert_eq!(zero_node_2nd_parameter.get_data(), None);
    assert_eq!(zero_node_2nd_parameter.get_string_data().unwrap(), "eleven");

    // That's everything
    assert_eq!(zero_node_2nd_parameter.get_next_node_index(), None);

    

    // But parameters passed to (= <a> <b>) should match the input type
    let eleven_is_greater_than_zero = &nodes[globals[2].get_first_node_index()];
    assert_eq!(eleven_is_greater_than_zero.get_value_type(), ValueType::Boolean); // return type

    // Function name
    let eleven_is_greater_than_zero_function_name = match eleven_is_greater_than_zero.get_data() {
        Some(NodeData::NodeOffset(n)) => &nodes[n],
        _ => unreachable!()
    };
    assert_eq!(eleven_is_greater_than_zero_function_name.get_value_type(), ValueType::FunctionName); // function name
    assert_eq!(eleven_is_greater_than_zero_function_name.get_string_data().unwrap(), "=");

    // First parameter - 5
    let eleven_is_greater_than_zero_1st_parameter = &nodes[eleven_is_greater_than_zero_function_name.get_next_node_index().unwrap()];
    assert_eq!(eleven_is_greater_than_zero_1st_parameter.get_value_type(), ValueType::Short);
    assert_eq!(eleven_is_greater_than_zero_1st_parameter.get_data(), None);
    assert_eq!(eleven_is_greater_than_zero_1st_parameter.get_string_data().unwrap(), "eleven");

    // Second parameter - 6
    let eleven_is_greater_than_zero_2nd_parameter = &nodes[eleven_is_greater_than_zero_1st_parameter.get_next_node_index().unwrap()];
    assert_eq!(eleven_is_greater_than_zero_2nd_parameter.get_value_type(), ValueType::Short);
    assert_eq!(eleven_is_greater_than_zero_2nd_parameter.get_data(), None);
    assert_eq!(eleven_is_greater_than_zero_2nd_parameter.get_string_data().unwrap(), "zero");

    // That's everything
    assert_eq!(eleven_is_greater_than_zero_2nd_parameter.get_next_node_index(), None);
}
