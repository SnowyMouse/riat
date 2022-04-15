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
    compiler.compile_script_data().unwrap();

    // Let's look at each global
    assert_eq!(compiler.globals.len(), 3);

    // Literal parameters passed to (+ <a> <b>) should be real
    let eleven_node = compiler.globals[0].node.as_ref().unwrap();
    assert_eq!(eleven_node.value_type, ValueType::Short);
    assert_eq!(eleven_node.parameters.as_ref().unwrap()[0].value_type, ValueType::Real);

    // Globals passed to (+ <a> <b>) should be real even if short
    let zero_node = compiler.globals[1].node.as_ref().unwrap();
    assert_eq!(zero_node.value_type, ValueType::Short);
    assert_eq!(zero_node.parameters.as_ref().unwrap()[0].value_type, ValueType::Real);

    // But parameters passed to (= <a> <b>) should match the input type
    let eleven_is_greater_than_zero_node = compiler.globals[2].node.as_ref().unwrap();
    assert_eq!(eleven_is_greater_than_zero_node.value_type, ValueType::Boolean);
    assert_eq!(eleven_is_greater_than_zero_node.parameters.as_ref().unwrap()[0].value_type, ValueType::Short);
}
