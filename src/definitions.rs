extern crate hiat_definitions;
use self::hiat_definitions::generate_definitions;
use super::ValueType;

pub struct EngineAvailability {
    pub mcc_cea: Option<u16>,
    pub gbx_retail: Option<u16>,
    pub gbx_custom: Option<u16>,
    pub gbx_demo: Option<u16>,
    pub xbox_ntsc: Option<u16>
}

pub struct EngineFunctionParameter {
    pub value_type: ValueType,
    pub many: bool,
    pub many_group: bool,
    pub allow_uppercase: bool
}

pub struct EngineFunction {
    pub name: &'static str,
    pub parameters: &'static [EngineFunctionParameter],
    pub number_passthrough: bool,
    pub return_type: ValueType,
    pub availability: EngineAvailability
}

pub trait CallableFunction {
    /// Get the function name.
    fn get_name(&self) -> &str;

    /// Get the return type.
    fn get_return_type(&self) -> ValueType;

    /// Get the number of parameters of the function.
    fn get_parameter_count(&self) -> usize;

    /// Get the type of a given parameter, taking into account parameters that can be used multiple times.
    fn get_type_of_parameter(&self, index: usize) -> Option<ValueType>;

    /// Get whether any 'real' function parameters can be converted to any other numerical type.
    fn is_number_passthrough(&self) -> bool;
}

impl CallableFunction for EngineFunction {
    fn get_name(&self) -> &str {
        self.name
    }

    fn get_return_type(&self) -> ValueType {
        self.return_type
    }

    fn get_parameter_count(&self) -> usize {
        self.parameters.len()
    }

    fn get_type_of_parameter(&self, index: usize) -> Option<ValueType> {
        match self.parameters.len() {
            // No parameters
            0 => None,

            // We are within the parameters range
            n if index < n => Some(self.parameters[index].value_type),

            // We are outside of the range
            n => {
                let last_parameter_index = n - 1;
                let last_parameter = &self.parameters[last_parameter_index];

                // Groups
                if last_parameter.many_group {
                    // Find the first parameter with many_group
                    let first_parameter_many_group_index = self.parameters.iter().position(|x| { x.many_group }).unwrap();

                    // Get the expected parameter
                    Some(self.parameters[(index - first_parameter_many_group_index) % (last_parameter_index - first_parameter_many_group_index)].value_type)
                }

                // Repeat the last one
                else if last_parameter.many {
                    Some(last_parameter.value_type)
                }

                // No multi-parameters {
                else {
                    None
                }
            }
        }
    }

    fn is_number_passthrough(&self) -> bool {
        self.number_passthrough
    }
}

pub struct EngineGlobal {
    pub name: &'static str,
    pub value_type: ValueType,
    pub availability: EngineAvailability
}

pub trait CallableGlobal {
    /// Get the name of the global.
    fn get_name(&self) -> &str;

    /// Get the value type of the global.
    fn get_value_type(&self) -> ValueType;
}

impl CallableGlobal for EngineGlobal {
    fn get_name(&self) -> &str {
        self.name
    }

    fn get_value_type(&self) -> ValueType {
        self.value_type
    }
}

generate_definitions!();
