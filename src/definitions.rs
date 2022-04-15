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
    allow_uppercase: bool,
    optional: bool
}

pub(crate) struct EngineFunction {
    pub name: &'static str,
    pub parameters: &'static [EngineFunctionParameter],
    pub number_passthrough: bool,
    pub passthrough_last: bool,
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

    fn get_total_parameter_count(&self) -> usize {
        self.parameters.len()
    }

    fn get_minimum_parameter_count(&self) -> usize {
        let parameter_count = self.parameters.len();

        for i in 0..parameter_count {
            let parameter = &self.parameters[i];

            if parameter.optional {
                return i
            }
        }

        parameter_count
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

                // Repeat the last one
                if last_parameter.many {
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

    fn is_engine_function(&self) -> bool {
        true
    }

    fn is_passthrough_last(&self) -> bool {
        self.passthrough_last
    }

    fn is_uppercase_allowed_for_parameter(&self, parameter_index: usize) -> bool {
        if parameter_index < self.parameters.len() {
            self.parameters[parameter_index].allow_uppercase
        }
        else {
            false
        }
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

    fn is_engine_global(&self) -> bool {
        true
    }
}

generate_definitions!();
