use super::*;
use std::fmt::Display;

/// Compile target to use. This determines available features, such as functions.
#[derive(Copy, Clone, PartialEq)]
#[repr(C)]
pub enum CompileTarget {
    /// Halo: Combat Evolved Anniversary as released by 343 Industries for Windows.
    HaloCEA,

    /// NTSC Xbox version is supported
    HaloCEXboxNTSC,

    /// Halo: Combat Evolved as released by Gearbox and MacSoft on Windows and Mac OS X, respectively.
    ///
    /// This also applies to the demo released by MacSoft.
    HaloCEGBX,

    /// Halo: Combat Evolved demo as released by Gearbox on Windows.
    ///
    /// This also applies to the un-updated CD version by Gearbox on Windows.
    ///
    /// This does not apply to the demo released by MacSoft for Mac OS X, as it's based on a newer version.
    HaloCEGBXDemo,

    /// Halo Custom Edition as released by Gearbox on Windows.
    HaloCustomEdition,
}

impl CompileTarget {
    /// Get the maximum number of script parameters supported for the target engine.
    pub fn maximum_script_parameters(&self) -> usize {
        match *self {
            CompileTarget::HaloCEA => 16,
            _ => 0
        }
    }
}

impl Display for CompileTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let name = match *self {
            CompileTarget::HaloCEA => "Halo: Combat Evolved Anniversary",
            CompileTarget::HaloCustomEdition => "Halo: Custom Edition",
            CompileTarget::HaloCEGBX => "Halo: Combat Evolved (PC / Mac)",
            CompileTarget::HaloCEGBXDemo => "Halo: Combat Evolved Demo (PC / Mac)",
            CompileTarget::HaloCEXboxNTSC => "Halo: Combat Evolved (Xbox)",
        };
        f.write_str(name)
    }
}

/// Script type which determines how a script is run and parsed.
#[derive(PartialEq, Copy, Clone)]
#[repr(C)]
pub enum ScriptType {
    /// Script called on startup.
    ///
    /// This script always returns void.
    Startup,

    /// Continuous script that can be awoken later.
    ///
    /// This script always returns void.
    Dormant,

    /// Script called every tick.
    ///
    /// This script always returns void.
    Continuous,

    /// Script which can be called manually.
    Static,

    /// Static script that can be replaced by a non-stub script later.
    Stub
}

impl ScriptType {
    /// Get whether or not the script type always returns void and does not have a type
    pub fn always_returns_void(&self) -> bool {
        *self != ScriptType::Static && *self != ScriptType::Stub
    }

    /// Get the offset to the expression tokens
    pub fn expression_offset(&self) -> usize {
        3 + if self.always_returns_void() {0} else {1}
    }

    /// Get the script type from a string (as used in HSC)
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

    /// Convert the script type to string (as used in HSC)
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

/// Scripts are custom functions that can be called by globals, other scripts, or sometimes automatically.
pub(crate) struct Script {
    /// Name of the script
    pub name: String,

    /// Return type of the script
    pub return_type: ValueType,

    /// Type of the script
    pub script_type: ScriptType,

    /// Token of the script (internal only)
    pub(crate) original_token: Token,

    /// Node of the script
    pub node: Node,

    /// Parameters of the script
    pub parameters: Vec<ScriptParameter>
}

impl CallableFunction for Script {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_return_type(&self) -> ValueType {
        self.return_type
    }

    fn supports_target(&self, _target: CompileTarget) -> bool {
        true
    }

    fn get_total_parameter_count(&self) -> usize {
        self.parameters.len()
    }

    fn get_type_of_parameter(&self, index: usize) -> Option<ValueType> {
        match self.parameters.get(index) {
            Some(n) => Some(n.value_type),
            None => None
        }
    }
}

/// Script parameters are part of scripts and work like globals.
pub(crate) struct ScriptParameter {
    /// Name of the parameter
    pub name: String,

    /// Type of the parameter
    pub value_type: ValueType,

    /// Token of the global (internal only)
    pub(crate) original_token: Token
}

impl CallableGlobal for ScriptParameter {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_value_type(&self) -> ValueType {
        self.value_type
    }

    fn supports_target(&self, _target: CompileTarget) -> bool {
        true
    }
}

/// Globals are custom variables that can be used by other globals and scripts.
pub(crate) struct Global {
    /// Name of the global
    pub name: String,

    /// Value type of the global
    pub value_type: ValueType,

    /// Token of the global (internal only)
    pub(crate) original_token: Token,

    /// Node of the global
    pub node: Node
}

impl CallableGlobal for Global {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_value_type(&self) -> ValueType {
        self.value_type
    }

    fn supports_target(&self, _target: CompileTarget) -> bool {
        true
    }
}

/// Function that can be called in a script
#[allow(unused_variables)]
pub(crate) trait CallableFunction {
    /// Get the function name.
    fn get_name(&self) -> &str;

    /// Get the return type.
    fn get_return_type(&self) -> ValueType;

    /// Get the maximum number of required parameters of the function.
    fn get_total_parameter_count(&self) -> usize {
        0
    }

    /// Get the minimum number of required parameters of the function.
    fn get_minimum_parameter_count(&self) -> usize {
        self.get_total_parameter_count()
    }

    /// Get the type of a given parameter, taking into account parameters that can be used multiple times.
    fn get_type_of_parameter(&self, index: usize) -> Option<ValueType> {
        None
    }

    /// Get whether the passthrough type must be numerical.
    fn is_number_passthrough(&self) -> bool {
        false
    }

    /// Get whether the last parameter is used for determining passthrough.
    fn is_passthrough_last(&self) -> bool {
        false
    }

    /// Get whether the function is an inequality function.
    fn is_inequality(&self) -> bool {
        false
    }

    /// Get whether or not the target engine is supported
    fn supports_target(&self, target: CompileTarget) -> bool;

    /// Get whether or not a parameter at a given index can use an uppercase literal
    fn is_uppercase_allowed_for_parameter(&self, parameter_index: usize) -> bool {
        false
    }

    /// Get whether or not it's an engine function
    fn is_engine_function(&self) -> bool {
        false
    }
}

/// Global that can be referenced in a script
pub(crate) trait CallableGlobal {
    /// Get the name of the global.
    fn get_name(&self) -> &str;

    /// Get the value type of the global.
    fn get_value_type(&self) -> ValueType;

    /// Get whether or not the target engine is supported
    fn supports_target(&self, target: CompileTarget) -> bool;

    /// Get whether or not it's an engine global
    fn is_engine_global(&self) -> bool {
        false
    }
}

/// Data of the node.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum NodeData {
    Boolean(bool),
    Short(i16),
    Long(i32),
    Real(f32),
    NodeOffset(usize)
}

/// Primitive type
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PrimitiveType {
    /// Value is a static value.
    Static,

    /// Value is a local variable.
    Local,

    /// Value is a global variable.
    Global
}

/// Type of the node.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum NodeType {
    /// Node refers to a value.
    ///
    /// If PrimitiveType refers to a variable, then string_data will be set to the variable name.
    Primitive(PrimitiveType),

    /// Node is a function call.
    ///
    /// If the boolean is `true`, then the function is an engine function.
    FunctionCall(bool)
}

impl Default for NodeType {
    fn default() -> Self {
        Self::Primitive(PrimitiveType::Static)
    }
}

impl NodeType {
    /// Get whether or not the node type is a variable.
    pub fn is_variable(&self) -> bool {
        *self == NodeType::Primitive(PrimitiveType::Local) || *self == NodeType::Primitive(PrimitiveType::Global)
    }

    /// Get whether or not the node type is a static value and not a variable.
    pub fn is_static_value(&self) -> bool {
        *self == NodeType::Primitive(PrimitiveType::Static)
    }

    /// Get whether or not the node type is an engine function.
    pub fn is_engine_function(&self) -> bool {
        *self == NodeType::FunctionCall(true)
    }

    /// Get whether or not the node type is a script.
    pub fn is_script(&self) -> bool {
        *self == NodeType::FunctionCall(false)
    }

    /// Get whether or not the node type is a primitive.
    pub fn is_primitive(&self) -> bool {
        matches!(*self, NodeType::Primitive(_))
    }

    /// Get whether or not the node type is a function call.
    pub fn is_function_call(&self) -> bool {
        matches!(*self, NodeType::FunctionCall(_))
    }
}

/*
use std::ffi::{CString, CStr};

impl CompileEncoding {
    /// Encode to a null-terminated C string.
    pub fn encode_to_cstring(&self, string: &str) -> CString {
        match *self {
            CompileEncoding::UTF8 => {
                CString::new(string).unwrap()
            },
            CompileEncoding::Windows1252 => {
                CString::new(WINDOWS_1252.encode(string, EncoderTrap::Replace).unwrap()).unwrap()
            }
        }
    }

    /// Decode the string from a C string.
    ///
    /// # Errors
    ///
    /// Errors if an error occurred on decoding.
    pub fn decode_from_cstring(&self, string: &CStr) -> Result<String, String> {
        self.decode_from_bytes(string.to_bytes())
    }

    /// Decode the string from an array of bytes.
    ///
    /// # Errors
    ///
    /// Errors if an error occurred on decoding.
    pub fn decode_from_bytes(&self, string: &[u8]) -> Result<String, String> {
        match *self {
            CompileEncoding::UTF8 => {
                match std::str::from_utf8(string) {
                    Ok(n) => Ok(n.to_owned()),
                    Err(e) => Err(format!("{e:?}"))
                }
            },
            CompileEncoding::Windows1252 => {
                match WINDOWS_1252.decode(string, DecoderTrap::Replace) {
                    Ok(n) => Ok(n),
                    Err(e) => Err(format!("{e:?}"))
                }
            }
        }
    }
}
*/
