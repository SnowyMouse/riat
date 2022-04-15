use super::*;

/// Result of a successful compilation
pub struct CompiledScriptData {
    pub(super) scripts: Vec<CompiledScript>,
    pub(super) globals: Vec<CompiledGlobal>,
    pub(super) files: Vec<String>,
    pub(super) warnings: Vec<CompileError>,
    pub(super) nodes: Vec<CompiledNode>
}

impl CompiledScriptData {
    /// Get all scripts that were compiled.
    pub fn get_scripts(&self) -> &[CompiledScript] {
        &self.scripts
    }

    /// Get all globals that were compiled.
    pub fn get_globals(&self) -> &[CompiledGlobal] {
        &self.globals
    }

    /// Get all files that were compiled.
    pub fn get_files(&self) -> &[String] {
        &self.files
    }

    /// Get all warnings from compiling.
    pub fn get_warning(&self) -> &[CompileError] {
        &self.warnings
    }

    /// Get all compiled nodes
    pub fn get_nodes(&self) -> &[CompiledNode] {
        &self.nodes
    }
}


/// Compiled script result.
pub struct CompiledScript {
    pub(super) name: String,
    pub(super) value_type: ValueType,
    pub(super) script_type: ScriptType,
    pub(super) first_node: usize
}

impl CompiledScript {
    /// Get the name of the script.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Get the return value type.
    pub fn get_value_type(&self) -> ValueType {
        self.value_type
    }

    /// Get the script type.
    pub fn get_type(&self) -> ScriptType {
        self.script_type
    }

    /// Get the index of the first node.
    pub fn get_first_node_index(&self) -> usize {
        self.first_node
    }
}


/// Compiled global result.
pub struct CompiledGlobal {
    pub(super) name: String,
    pub(super) value_type: ValueType,
    pub(super) first_node: usize
}

impl CompiledGlobal {
    /// Get the name of the global.
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Get the value type.
    pub fn get_value_type(&self) -> ValueType {
        self.value_type
    }

    /// Get the index of the first node.
    pub fn get_first_node_index(&self) -> usize {
        self.first_node
    }
}


/// Compiled node result.
pub struct CompiledNode {
    pub(super) node_type: NodeType,
    pub(super) value_type: ValueType,
    pub(super) data: Option<NodeData>,
    pub(super) string_data: Option<String>,
    pub(super) next_node: Option<usize>
}

impl CompiledNode {
    /// Get the type of node
    pub fn get_type(&self) -> NodeType {
        self.node_type
    }

    /// Get the return value type.
    pub fn get_value_type(&self) -> ValueType {
        self.value_type
    }

    /// Get the data of the node, if any.
    pub fn get_data(&self) -> Option<NodeData> {
        self.data
    }

    /// Get the string data of the node, if any.
    pub fn get_string_data(&self) -> Option<&str> {
        match self.string_data.as_ref() {
            Some(n) => Some(n.as_str()),
            None => None
        }
    }

    /// Get the next node index, if any.
    pub fn get_next_node_index(&self) -> Option<usize> {
        self.next_node
    }
}
