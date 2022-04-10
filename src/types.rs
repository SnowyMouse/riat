use super::*;

/// Compile target to use
#[derive(Copy, Clone, PartialEq)]
pub enum CompileTarget {
    /// Halo: Combat Evolved Anniversary as released by 343 Industries on Windows.
    HaloCombatEvolvedAnniversary,

    /// Halo: Combat Evolved as released by Gearbox and MacSoft on Windows and Mac OS X, respectively.
    ///
    /// This also applies to the demo released by MacSoft.
    GearboxHaloCombatEvolved,

    /// Halo: Combat Evolved demo as released by Gearbox on Windows.
    ///
    /// This also applies to the un-updated CD version by Gearbox on Windows.
    ///
    /// This does not apply to the demo released by MacSoft for Mac OS X, as it's based on a newer version.
    GearboxHaloCombatEvolvedDemo,

    /// Halo Custom Edition as released by Gearbox on Windows.
    GearboxHaloCustomEdition,
}

/// Script type which determines how a script is run and parsed.
#[derive(PartialEq, Copy, Clone)]
pub enum ScriptType {
    /// Script which can be called manually.
    Static,

    /// Static script that can be replaced by a non-stub script later.
    Stub,

    /// Script called every tick.
    ///
    /// This script always returns void.
    Continuous,

    /// Continuous script that can be awoken later.
    ///
    /// This script always returns void.
    Dormant,

    /// Script called on startup.
    ///
    /// This script always returns void.
    Startup
}

impl ScriptType {
    pub fn always_returns_void(&self) -> bool {
        *self != ScriptType::Static && *self != ScriptType::Stub
    }

    pub fn from_str(input: &str) -> Option<ScriptType> {
        match input {
            "static" => Some(ScriptType::Static),
            "stub" => Some(ScriptType::Stub),
            "continuous" => Some(ScriptType::Continuous),
            "dormant" => Some(ScriptType::Dormant),
            "startup" => Some(ScriptType::Startup),
            _ => None
        }
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            ScriptType::Static => "static",
            ScriptType::Stub => "stub",
            ScriptType::Continuous => "continuous",
            ScriptType::Dormant => "dormant",
            ScriptType::Startup => "startup",
        }
    }
}

pub(crate) struct Script {
    pub name: String,
    pub return_type: ValueType,
    pub script_type: ScriptType,

    pub original_token_offset: usize,
    pub original_body_offset: usize
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

pub(crate) struct Global {
    pub name: String,
    pub value_type: ValueType,

    pub original_token_offset: usize,
    pub original_body_offset: usize
}

/// Function that can be called in a script
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

/// Global that can be referenced in a script
pub trait CallableGlobal {
    /// Get the name of the global.
    fn get_name(&self) -> &str;

    /// Get the value type of the global.
    fn get_value_type(&self) -> ValueType;
}
