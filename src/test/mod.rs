use super::*;

#[test]
fn test_tokenizer() {
    let hello_world_hsc = include_bytes!("script/hello_world.hsc");
    let mut compiler = Compiler::new(CompileTarget::HaloCEA);
    compiler.read_script_data("hello_world.hsc", hello_world_hsc).unwrap();

    assert_eq!(compiler.tokens.len(), 1); // 1 script
    assert_eq!(compiler.tokens[0].children.as_ref().unwrap().len(), 5); // 5 elements in the script - 'script', 'static', 'void', 'hello_world', and the body
    assert_eq!(compiler.tokens[0].children.as_ref().unwrap()[4].children.as_ref().unwrap().len(), 2); // 2 elements in the body - 'print' and 'hello world'
}
