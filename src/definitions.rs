extern crate hiat_definitions;
use self::hiat_definitions::generate_definitions;
use super::ValueType;

pub struct Availability {
    pub mcc_cea: Option<u16>,
    pub gbx_retail: Option<u16>,
    pub gbx_custom: Option<u16>,
    pub gbx_demo: Option<u16>,
    pub xbox_ntsc: Option<u16>
}

pub struct FunctionParameter {
    pub value_type: ValueType,
    pub many: bool,
    pub many_group: bool
}

pub struct Function {
    pub name: &'static str,
    pub parameters: &'static [FunctionParameter],
    pub number_passthrough: bool,
    pub return_type: ValueType,
    pub availability: Availability
}

pub struct Global {
    pub name: &'static str,
    pub value_type: ValueType,
    pub availability: Availability
}

generate_definitions!();
