extern crate riat;
use riat::*;

use std::os::raw::*;
use std::ffi::CStr;


/// Compile error C struct.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CompileErrorC {
    /// Pointer to a null terminated string containing the file name.
    pub file: *const c_char,

    /// Pointer to a null terminated string containing the message.
    pub message: *const c_char,

    /// Line the error occured on.
    pub line: usize,

    /// Column the error occured on.
    pub column: usize,

    /// Reserved
    pub base: *mut CompileError
}

impl CompileErrorC {
    fn new_owned(error: CompileError) -> Self {
        let reserved_box = Box::new(error);
        let mut e = Self::new(&reserved_box);
        e.base = Box::into_raw(reserved_box);
        e
    }

    fn new(error: &CompileError) -> Self {
        let (line, column) = error.get_position();

        Self {
            file: error.get_file_cstr().as_ptr(),
            message: error.get_message_cstr().as_ptr(),
            line: line,
            column: column,
            base: std::ptr::null_mut()
        }
    }

    unsafe fn free(&mut self) {
        if !self.base.is_null() {
            Box::from_raw(self.base);
            self.base = std::ptr::null_mut();
            self.file = std::ptr::null();
            self.message = std::ptr::null();
            self.line = 0;
            self.column = 0;
        }
    }
}


/// Allocate a compiler instance with the given target and return a pointer to it.
///
/// # Requirements
///
/// The resulting pointer must be freed with [`riat_compiler_free`] or else a memory leak will occur.
///
/// The target must be a valid [`CompileTarget`] enum or else **undefined behavior** will occur.
#[no_mangle]
pub extern "C" fn riat_compiler_new(target: CompileTarget, encoding: CompileEncoding) -> *mut Compiler {
    Box::into_raw(Box::<Compiler>::new(Compiler::new(target, encoding)))
}

/// Free a Compiler instance.
///
/// # Requirements
///
/// If any of these requirements are not met, **undefined behavior** will occur:
/// * The `compiler` parameter must point to a valid [`Compiler`] or be null.
/// * If non-null, make sure the function you got the pointer from states that this function needs to be used to clean it up.
#[no_mangle]
pub unsafe extern "C" fn riat_compiler_free(compiler: *mut Compiler) {
    if !compiler.is_null() {
        Box::from_raw(compiler);
    }
}

/// Free an error returned by a function.
///
/// Anything pointed to by the [`CompileErrorC`] struct will no longer be valid.
///
/// # Requirements
///
/// If any of these requirements are not met, **undefined behavior** will occur:
/// * The `CompileErrorC` pointed to must be initialized either as a zeroed-out struct or from a riatc function.
/// * The function that initialized the `CompileErrorC` must state that this function needs to be used to clean it up.
#[no_mangle]
pub unsafe extern "C" fn riat_error_free(error: *mut CompileErrorC) {
    (*error).free()
}

/// Read tokens from the given file.
///
/// Returns zero on success.
///
/// # Errors
///
/// On failure, a nonzero number is returned, and, if `error` is non-null, the pointer pointed to by `error` will be set to the error.
///
/// # Requirements
///
/// If an error is returned, the resulting error must be freed with [`riat_error_free`] or else a memory leak will occur.
///
/// If any of these requirements are not met, **undefined behavior** will occur:
/// * `input_filename` must be valid, null-terminated string in the correct encoding or else a panic will occur which may result in UB.
/// * `input_data` must point to a region of size `input_data_length` (it does not need to be null-terminated).
/// * `error` must either be null or point to a writable region large enough to hold a pointer.
#[no_mangle]
pub unsafe extern "C" fn riat_compiler_read_script_data(compiler: *mut Compiler, input_filename: *const c_char, input_data: *const u8, input_data_length: usize, error: *mut CompileErrorC) -> c_int {
    let compiler_ref = &mut *compiler;
    let filename = compiler_ref.get_encoder().decode_from_cstring(CStr::from_ptr(input_filename)).unwrap();
    let input_data_slice = std::slice::from_raw_parts(input_data, input_data_length);

    match compiler_ref.read_script_data(filename.as_str(), input_data_slice) {
        Ok(()) => 0,
        Err(e) => {
            if !error.is_null() {
                *error = CompileErrorC::new_owned(e);
            }
            -1
        }
    }
}

/// Read tokens from the given file.
///
/// Returns zero on success.
///
/// # Errors
///
/// On failure, a null pointer is returned, and, if `error` is non-null, the pointer pointed to by `error` will be set to the error.
///
/// # Requirements
///
/// If the function succeeds, the resulting pointer must be freed with [`riat_script_data_free`] or else a memory leak will occur.
///
/// If an error is returned, the resulting error must be freed with [`riat_error_free`] or else a memory leak will occur.
#[no_mangle]
pub unsafe extern "C" fn riat_compiler_compile_script_data(compiler: *mut Compiler, error: *mut CompileErrorC) -> *mut CompiledScriptData {
    match (*compiler).compile_script_data() {
        Ok(n) => Box::into_raw(Box::new(n)),
        Err(e) => {
            if !error.is_null() {
                *error = CompileErrorC::new_owned(e);
            }
            std::ptr::null_mut()
        }
    }
}

/// Free script data.
///
/// # Requirements
///
/// If any of these requirements are not met, **undefined behavior** will occur:
/// * The `script_data` parameter must point to a valid [`CompiledScriptData`] or be null.
/// * If non-null, make sure the function you got the pointer from states that this function needs to be used to clean it up.
#[no_mangle]
pub unsafe extern "C" fn riat_script_data_free(script_data: *mut CompiledScriptData) {
    if !script_data.is_null() {
        Box::from_raw(script_data);
    }
}


/// Get all warnings from the script compilation.
///
/// Return the number of warnings. Write this many warnings to an array pointed to by `warnings` if `warnings` is non-null.
///
/// These warnings must NOT be freed with [`riat_error_free`], as the resources are owned by the [`CompiledScriptData`], not the [`CompileErrorC`] struct.
///
/// # Requirements
///
/// If any of these requirements are not met, **undefined behavior** will occur:
/// * The `script_data` parameter must point to a valid [`CompiledScriptData`].
/// * The `warnings` parameter must point to a valid array of [`CompileErrorC`] long enough to hold the result of this function or be null. To query the number of warnings, run this function with this parameter set to null.
/// * If [`riat_script_data_free`] is called, the resulting warnings will no longer be valid, thus no pointers may be dereferenced after this.
#[no_mangle]
pub unsafe extern "C" fn riat_script_data_get_warnings(script_data: *const CompiledScriptData, warnings: *mut CompileErrorC) -> usize {
    let all_warnings = (*script_data).get_warnings();
    let count = all_warnings.len();

    if !warnings.is_null() {
        for i in 0..count {
            *warnings.add(i) = CompileErrorC::new(&all_warnings[i])
        }
    }

    count
}

/// Node type C enum.
#[repr(C)]
#[derive(Copy, Clone)]
pub enum NodeTypeC {
    StaticValue,
    LocalVariable,
    GlobalVariable,
    FunctionCall,
    ScriptCall
}

impl NodeTypeC {
    fn new(t: NodeType) -> Self {
        match t {
            NodeType::Primitive(PrimitiveType::Local) => Self::LocalVariable,
            NodeType::Primitive(PrimitiveType::Global) => Self::GlobalVariable,
            NodeType::Primitive(PrimitiveType::Static) => Self::StaticValue,
            NodeType::FunctionCall(true) => Self::FunctionCall,
            NodeType::FunctionCall(false) => Self::ScriptCall
        }
    }
}

/// Script node data C union.
#[repr(C)]
#[derive(Copy, Clone)]
pub union ScriptNodeDataC {
    pub offset: usize,
    pub real: f32,
    pub long: i32,
    pub short: i16,
    pub boolean: bool
}

/// Script node C struct.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ScriptNodeC {
    /// Pointer to a null terminated string containing the file name for the node.
    pub file: *const c_char,

    /// Line the node occured on.
    pub line: usize,

    /// Column the node occured on.
    pub column: usize,

    /// Pointer to a null terminated string containing the string data if valid. Otherwise, this is null.
    pub string_data: *const c_char,

    /// Index union (equal to type unless it's a function call or function name)
    pub index_union: u16,

    /// Value type of the node
    pub value_type: ValueType,

    /// Type of the node
    pub node_type: NodeTypeC,

    /// Data of the node
    pub node_data: ScriptNodeDataC,

    /// Offset to the next node if valid. Otherwise, this will be set to [`usize::MAX`] (SIZE_MAX)
    pub next_node: usize,
}


/// Get all nodes from the script compilation.
///
/// Return the number of nodes. Write this many nodes to an array pointed to by `nodes` if `nodes` is non-null.
///
/// # Requirements
///
/// If any of these requirements are not met, **undefined behavior** will occur:
/// * The `script_data` parameter must point to a valid [`CompiledScriptData`].
/// * The `nodes` parameter must point to a valid array of [`ScriptNodeC`] long enough to hold the result of this function or be null. To query the number of warnings, run this function with this parameter set to null.
/// * If [`riat_script_data_free`] is called, the resulting nodes will no longer be valid, thus no pointers may be dereferenced after this.
#[no_mangle]
pub unsafe extern "C" fn riat_script_data_get_nodes(script_data: *const CompiledScriptData, nodes: *mut ScriptNodeC) -> usize {
    let all_nodes = (*script_data).get_nodes();
    let all_files = (*script_data).get_files();
    let count = all_nodes.len();

    if !nodes.is_null() {
        for i in 0..count {
            let node_out = &mut *nodes.add(i);
            let node_in = &all_nodes[i];

            node_out.file = all_files[node_in.get_file()].as_ptr();
            node_out.line = node_in.get_line();
            node_out.column = node_in.get_column();
            node_out.string_data = match node_in.get_string_data_cstr() {
                Some(n) => n.as_ptr(),
                None => std::ptr::null()
            };
            node_out.next_node = node_in.get_next_node_index().unwrap_or(usize::MAX);
            node_out.value_type = node_in.get_value_type();
            node_out.node_type = NodeTypeC::new(node_in.get_type());
            node_out.index_union = node_in.get_index().unwrap_or(node_out.value_type as u16);
            node_out.node_data = match node_in.get_data() {
                None => ScriptNodeDataC { offset: usize::MAX },
                Some(n) => match n {
                    NodeData::Long(v) => ScriptNodeDataC { long: v },
                    NodeData::Short(v) => ScriptNodeDataC { short: v },
                    NodeData::Boolean(v) => ScriptNodeDataC { boolean: v },
                    NodeData::Real(v) => ScriptNodeDataC { real: v },
                    NodeData::NodeOffset(v) => ScriptNodeDataC { offset: v }
                }
            };
        }
    }

    count
}


/// Global C struct.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RIATGlobalC {
    /// Name of the global
    pub name: *const c_char,

    /// Pointer to a null terminated string containing the file name for the global.
    pub file: *const c_char,

    /// Line the global occured on.
    pub line: usize,

    /// Column the global occured on.
    pub column: usize,

    /// Value type of the global
    pub value_type: ValueType,

    /// Index of the first node
    pub first_node: usize
}

/// Script C struct.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RIATScriptC {
    /// Name of the script
    pub name: *const c_char,

    /// Pointer to a null terminated string containing the file name for the script.
    pub file: *const c_char,

    /// Line the script occured on.
    pub line: usize,

    /// Column the script occured on.
    pub column: usize,

    /// Type of the script
    pub script_type: ScriptType,

    /// Return type of the script
    pub return_type: ValueType,

    /// Index of the first node
    pub first_node: usize
}

/// Get all scripts from the script compilation.
///
/// Return the number of scripts. Write this many scripts to an array pointed to by `scripts` if `scripts` is non-null.
///
/// # Requirements
///
/// If any of these requirements are not met, **undefined behavior** will occur:
/// * The `script_data` parameter must point to a valid [`CompiledScriptData`].
/// * The `scripts` parameter must point to a valid array of [`RIATScriptC`] long enough to hold the result of this function or be null. To query the number of warnings, run this function with this parameter set to null.
/// * If [`riat_script_data_free`] is called, the resulting scripts will no longer be valid, thus no pointers may be dereferenced after this.
#[no_mangle]
pub unsafe extern "C" fn riat_script_data_get_scripts(script_data: *const CompiledScriptData, scripts: *mut RIATScriptC) -> usize {
    let all_scripts = (*script_data).get_scripts();
    let all_files = (*script_data).get_files();
    let count = all_scripts.len();

    if !scripts.is_null() {
        for i in 0..count {
            let script_out = &mut *scripts.add(i);
            let script_in = &all_scripts[i];

            script_out.file = all_files[script_in.get_file()].as_ptr();
            script_out.line = script_in.get_line();
            script_out.column = script_in.get_column();
            script_out.name = script_in.get_name_cstr().as_ptr();
            script_out.first_node = script_in.get_first_node_index();
            script_out.return_type = script_in.get_value_type();
            script_out.script_type = script_in.get_type();
        }
    }

    count
}

/// Script parameter C struct.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RIATScriptParameterC {
    /// Name of the script
    pub name: *const c_char,

    /// Type of the parameter
    pub value_type: ValueType
}

/// Get all script parameters from the script compilation for the script.
///
/// Return the number of script parameters. Write this many script parameters to an array pointed to by `parameters` if `parameters` is non-null.
///
/// Return 0 and do nothing if the script was not found.
///
/// # Requirements
///
/// If any of these requirements are not met, **undefined behavior** will occur:
/// * The `script_data` parameter must point to a valid [`CompiledScriptData`].
/// * The `script_name` must point to a valid null-terminated C string.
/// * The `scripts` parameter must point to a valid array of [`RIATScriptC`] long enough to hold the result of this function or be null. To query the number of warnings, run this function with this parameter set to null.
/// * If [`riat_script_data_free`] is called, the resulting scripts will no longer be valid, thus no pointers may be dereferenced after this.
#[no_mangle]
pub unsafe extern "C" fn riat_script_data_get_script_parameters(script_data: *const CompiledScriptData, script_name: *const c_char, parameters: *mut RIATScriptParameterC) -> usize {
    let all_scripts = (*script_data).get_scripts();
    let script_name = CStr::from_ptr(script_name).to_str().unwrap();

    for i in all_scripts {
        if i.get_name_cstr().to_str().unwrap() == script_name {
            let all_parameters = i.get_parameters();
            let count = all_parameters.len();

            if !parameters.is_null() {
                for c in 0..count {
                    let parameter_out = &mut *parameters.add(c);
                    let parameter_in = &all_parameters[c];
                    parameter_out.name = parameter_in.get_name_cstr().as_ptr();
                    parameter_out.value_type = parameter_in.get_value_type();
                }
            }

            return count;
        }
    }

    0
}

/// Get all globals from the global compilation.
///
/// Return the number of globals. Write this many globals to an array pointed to by `globals` if `globals` is non-null.
///
/// # Requirements
///
/// If any of these requirements are not met, **undefined behavior** will occur:
/// * The `global_data` parameter must point to a valid [`CompiledScriptData`].
/// * The `globals` parameter must point to a valid array of [`RIATScriptC`] long enough to hold the result of this function or be null. To query the number of warnings, run this function with this parameter set to null.
/// * If [`riat_script_data_free`] is called, the resulting globals will no longer be valid, thus no pointers may be dereferenced after this.
#[no_mangle]
pub unsafe extern "C" fn riat_script_data_get_globals(global_data: *const CompiledScriptData, globals: *mut RIATGlobalC) -> usize {
    let all_globals = (*global_data).get_globals();
    let all_files = (*global_data).get_files();
    let count = all_globals.len();

    if !globals.is_null() {
        for i in 0..count {
            let global_out = &mut *globals.add(i);
            let global_in = &all_globals[i];

            global_out.file = all_files[global_in.get_file()].as_ptr();
            global_out.line = global_in.get_line();
            global_out.column = global_in.get_column();
            global_out.name = global_in.get_name_cstr().as_ptr();
            global_out.first_node = global_in.get_first_node_index();
            global_out.value_type = global_in.get_value_type();
        }
    }

    count
}
