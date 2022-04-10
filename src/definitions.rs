extern crate hiat_definitions;
use self::hiat_definitions::generate_definitions;
use super::{ValueType, CallableGlobal, CallableFunction, CompileTarget};

pub(crate) struct EngineAvailability {
    pub mcc_cea: Option<u16>,
    pub gbx_retail: Option<u16>,
    pub gbx_custom: Option<u16>,
    pub gbx_demo: Option<u16>,
    pub xbox_ntsc: Option<u16>
}

impl EngineAvailability {
    fn supports(&self, target: CompileTarget) -> bool {
        match target {
            CompileTarget::HaloCEA => !matches!(self.mcc_cea, None),
            CompileTarget::HaloCEXboxNTSC => !matches!(self.xbox_ntsc, None),
            CompileTarget::HaloCEGBX => !matches!(self.gbx_retail, None),
            CompileTarget::HaloCEGBXDemo => !matches!(self.gbx_demo, None),
            CompileTarget::HaloCustomEdition => !matches!(self.gbx_custom, None),
        }
    }
}

pub(crate) struct EngineFunctionParameter {
    value_type: ValueType,
    many: bool,
    many_group: bool,
    allow_uppercase: bool
}

pub(crate) struct EngineFunction {
    pub name: &'static str,
    pub parameters: &'static [EngineFunctionParameter],
    pub number_passthrough: bool,
    pub return_type: ValueType,
    pub availability: EngineAvailability
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

    fn supports_target(&self, target: CompileTarget) -> bool {
        self.availability.supports(target)
    }
}

pub(crate) struct EngineGlobal {
    pub name: &'static str,
    pub value_type: ValueType,
    pub availability: EngineAvailability
}

impl CallableGlobal for EngineGlobal {
    fn get_name(&self) -> &str {
        self.name
    }

    fn get_value_type(&self) -> ValueType {
        self.value_type
    }

    fn supports_target(&self, target: CompileTarget) -> bool {
        self.availability.supports(target)
    }
}

generate_definitions!();
