pub enum ScriptType {
    Stub,
    Static,
    Continuous,
    Startup,
    Dormant
}

pub struct Script {
    name: String,
    return_type: ValueType,
    script_type: ScriptType,

    original_token_offset: usize,

}

impl CallableFunction for Script {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_return_type(&self) -> ValueType {
        self.return_type
    }

    fn get_parameter_count(&self) -> usize {
        0
    }

    fn get_type_of_parameter(&self, _index: usize) -> Option<ValueType> {
        None
    }

    fn is_number_passthrough(&self) -> bool {
        false
    }
}

pub struct Global {

}

use super::*;

impl Compiler {
    pub fn digest_tokens(&self) -> Result<(), CompileError> {
        todo!()
    }
}
