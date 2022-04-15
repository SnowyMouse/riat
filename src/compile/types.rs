use super::*;

/// Result of a successful compilation.
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
    pub(super) first_node: usize,

    pub(super) file: usize,
    pub(super) line: usize,
    pub(super) column: usize
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

    /// Get the file index of the script, starting at 0.
    ///
    /// This corresponds to [`CompiledScriptData::get_files`].
    pub fn get_file(&self) -> usize {
        self.file
    }

    /// Get the line index of the script, starting at 1.
    pub fn get_line(&self) -> usize {
        self.line
    }

    /// Get the column index of the script, starting at 1.
    pub fn get_column(&self) -> usize {
        self.column
    }
}


/// Compiled global result.
pub struct CompiledGlobal {
    pub(super) name: String,
    pub(super) value_type: ValueType,
    pub(super) first_node: usize,

    pub(super) file: usize,
    pub(super) line: usize,
    pub(super) column: usize
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

    /// Get the file index of the global, starting at 0.
    ///
    /// This corresponds to [`CompiledScriptData::get_files`].
    pub fn get_file(&self) -> usize {
        self.file
    }

    /// Get the line index of the global, starting at 1.
    pub fn get_line(&self) -> usize {
        self.line
    }

    /// Get the column index of the global, starting at 1.
    pub fn get_column(&self) -> usize {
        self.column
    }
}


/// Compiled node result.
pub struct CompiledNode {
    pub(super) node_type: NodeType,
    pub(super) value_type: ValueType,
    pub(super) data: Option<NodeData>,
    pub(super) string_data: Option<String>,
    pub(super) next_node: Option<usize>,

    pub(super) file: usize,
    pub(super) line: usize,
    pub(super) column: usize
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

    /// Get the file index of the node, starting at 0.
    ///
    /// This corresponds to [`CompiledScriptData::get_files`].
    pub fn get_file(&self) -> usize {
        self.file
    }

    /// Get the line index of the node, starting at 1.
    pub fn get_line(&self) -> usize {
        self.line
    }

    /// Get the column index of the node, starting at 1.
    pub fn get_column(&self) -> usize {
        self.column
    }
}


/// Data unit used for scripts.
#[derive(PartialEq, Clone, Debug, Default)]
pub(crate) struct Node {
    /// Value type
    pub value_type: ValueType,

    /// Node type
    pub node_type: NodeType,

    /// String data
    pub string_data: Option<String>,

    /// Node data
    pub data: Option<NodeData>,

    /// Parameters of the node (if a function call)
    pub parameters: Option<Vec<Node>>,

    /// File index the node is found on
    pub file: usize,

    /// Line the node is found on
    pub line: usize,

    /// Column the node is found on
    pub column: usize
}
