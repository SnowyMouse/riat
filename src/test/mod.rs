#[test]
fn test_tokenizer() {
    let hello_world_hsc = include_bytes!("script/hello_world.hsc");
    let mut compiler = super::Compiler::new();
    compiler.read_script_data("hello_world.hsc", hello_world_hsc).unwrap();

    assert_eq!(compiler.tokens.len(), 10);
}
