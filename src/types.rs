use super::*;

/// Compile target to use
#[derive(Copy, Clone, PartialEq)]
pub enum CompileTarget {
    /// Halo: Combat Evolved Anniversary as released by 343 Industries on Windows.
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
    pub original_body_offset: usize,

    pub node: Option<Node>
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
}

pub(crate) struct Global {
    pub name: String,
    pub value_type: ValueType,

    pub original_token_offset: usize,
    pub original_body_offset: usize,

    pub node: Option<Node>
}

impl CallableGlobal for Global {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_value_type(&self) -> ValueType {
        self.value_type
    }

    fn supports_target(&self, target: CompileTarget) -> bool {
        true
    }
}

/// Function that can be called in a script
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

    /// Get whether any 'real' function parameters can be converted to any other numerical type.
    fn is_number_passthrough(&self) -> bool {
        false
    }

    /// Get whether or not the target engine is supported
    fn supports_target(&self, target: CompileTarget) -> bool;

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

/// Node data
#[derive(PartialEq, Copy, Clone)]
pub enum NodeData {
    None,
    Boolean(bool),
    Short(i16),
    Long(i32),
    Real(f32)
}

/// Node type
#[derive(PartialEq, Copy, Clone)]
pub enum NodeType {
    /// Node refers to a value.
    ///
    /// If the boolean is true, then string_data will be set to the global name.
    Primitive(bool),

    /// Node is a function call.
    ///
    /// If the boolean is true, then the function is a script.
    FunctionCall(bool)
}

impl NodeType {
    /// Get whether or not the node type is a global.
    pub fn is_global(&self) -> bool {
        *self == NodeType::Primitive(true)
    }

    /// Get whether or not the node type is a not a global but is a primitive.
    pub fn is_nonglobal_primitive(&self) -> bool {
        *self == NodeType::Primitive(false)
    }
    
    /// Get whether or not the node type is a script.
    pub fn is_script(&self) -> bool {
        *self == NodeType::FunctionCall(true)
    }
    
    /// Get whether or not the node type is an engine function.
    pub fn is_engine_function(&self) -> bool {
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

/// Node stuff
#[derive(PartialEq, Clone)]
pub struct Node {
    /// Value type
    pub value_type: ValueType,

    /// Node type
    pub node_type: NodeType,

    /// String data
    pub string_data: Option<String>,

    /// Node data
    pub data: NodeData,

    /// Parameters of the node (if a function call)
    pub parameters: Option<Vec<Node>>
}