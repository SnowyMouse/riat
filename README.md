# Rat In a Tube
RIAT is a free script compiler for Halo: Combat Evolved scripts.

To compile, you will need the Rust compiler and Cargo.

## Example usage
Rat in a Tube can be used in Rust, C, and C++.

### Rust
RIAT is a Rust library, thus you can directly use it in your Rust library.

```rust
extern crate riat;
use riat::Compiler;

/** Return true if successful. Return false on failure. */
fn compile_scripts_rust(script_file_name: &str, script_data: &[u8]) -> bool {
    // Instantiate our instance
    let mut compiler = Compiler::new(CompileTarget::HaloCEA, CompileEncoding::Windows1252);
    
    // Try to compile
    match compiler.read_script_data(script_file_name, script_data) {
        Ok(_) => (),
        Err(e) => {
            let file = e.get_file();
            let message = e.get_message();
            let (line,column) = e.get_position();
            eprintln!("Failed to read script data: {file}:{line}:{column}: {message}");
            return false;
        }
    }
    let script_data = match compiler.digest_tokens() {
        Ok(n) => n,
        Err(e) => {
            let file = e.get_file();
            let message = e.get_message();
            let (line,column) = e.get_position();
            eprintln!("Failed to compile script data: {file}:{line}:{column}: {message}");
            return false;
        }
    };
    
    // Print information
    println!("Script count: {}", script_data.get_scripts().len());
    println!("Global count: {}", script_data.get_globals().len());
    println!("Node count: {}", script_data.get_nodes().len());
    
    return true;
}
```


### C
RIAT comes with C bindings via the riatc package. You will need to link with the
riatc library, compiled using your toolchain of choice, and have riatc's include
directory in your include list.

```c
#include <riat/riat.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>
#include <string.h>

/* Return 0 if successful. Return a nonzero value if failed. */
int compile_scripts_c(const char *script_file_name, const uint8_t *script_data, size_t script_data_length) {
    /* Put variables here */
    int result = 0;
    RIATCompiledScriptData *compiled_script_data = NULL;
    RIATScriptC *scripts = NULL;
    RIATGlobalC *globals = NULL;
    RIATScriptNodeC *nodes = NULL;
    size_t script_count, global_count, node_count;

    /* Zero-out error struct. Not required, but calling riat_error_free() on a zeroed out struct is always OK. */
    RIATCompileErrorC error;
    memset(&error, 0, sizeof(error));

    /* Instantiate our instance */
    RIATCompiler *compiler = riat_compiler_new(RIAT_HaloCEA, RIAT_Windows1252);
    
    /* Try to compile */
    if(riat_compiler_read_script_data(compiler, script_file_name, script_data, script_data_length, &error) != 0) {
        fprintf(stderr, "Failed to read script data: %s:%zu:%zu: %s\n", error.file, error.line, error.column, error.message);
        result = 1;
        goto CLEANUP;
    }
    if((compiled_script_data = riat_compiler_compile_script_data(compiler, &error)) == NULL) {
        fprintf(stderr, "Failed to compile script data: %s:%zu:%zu: %s\n", error.file, error.line, error.column, error.message);
        result = 1;
        goto CLEANUP;
    }
    
    /* Get the sizes */
    script_count = riat_script_data_get_scripts(compiled_script_data, NULL);
    global_count = riat_script_data_get_globals(compiled_script_data, NULL);
    node_count = riat_script_data_get_nodes(compiled_script_data, NULL);
    
    /* Allocate */
    scripts = malloc(sizeof(*scripts) * script_count);
    globals = malloc(sizeof(*globals) * global_count);
    nodes = malloc(sizeof(*nodes) * node_count);
    
    /* If any of those returned false, we failed */
    if(scripts == NULL || globals == NULL || nodes == NULL) {
        fprintf(stderr, "Allocation error!\n");
        result = 2;
        goto CLEANUP;
    }
    
    /* Read everything */
    riat_script_data_get_scripts(compiled_script_data, scripts);
    riat_script_data_get_globals(compiled_script_data, globals);
    riat_script_data_get_nodes(compiled_script_data, nodes);
    
    /* Print information */
    printf("Script count: %zu\n", script_count);
    printf("Global count: %zu\n", global_count);
    printf("Node count %zu\n", node_count);
    
    /* Cleanup */
    CLEANUP:
    riat_error_free(&error);
    riat_compiler_free(compiler);
    riat_script_data_free(compiled_script_data);
    free(scripts);
    free(globals);
    free(nodes);
    
    /* Done! */
    return result;
} 
```

### C++
The rustc package also contains a C++ header. You will need to link with the
riatc library, compiled using your toolchain of choice, and have riatc's include
directory in your include list.

While you can use the C header with your C++ library, the C++ header provides
exceptions, type safety, and automatic memory management through containers
such as std::vector.

```cpp
#include <riat/riat.hpp>
#include <iostream>
#include <cstdint>
#include <vector>

// Return true if successful. Return false on failure.
bool compile_scripts_cpp(const char *script_file_name, const std::vector<std::uint8_t> &script_data) {
    // Instantiate our instance
    RIAT::Compiler instance(RIATCompileTarget::RIAT_HaloCEA);
    RIAT::CompilerScriptResult result;
    
    // Try to compile
    try {
        instance.read_script_data(script_data.data(), script_data.size(), script_file_name);
        result = instance.compile_scripts();
    }
    catch(std::exception &e) {
        std::cerr << "Error when compiling: " << e.what() << "\n";
        return false;
    }
    
    // Print information
    auto scripts = result.get_scripts();
    auto globals = result.get_globals();
    auto nodes = result.get_nodes();
    std::cout << "Script count:" << scripts.size() << std::endl;
    std::cout << "Global count:" << globals.size() << std::endl;
    std::cout << "Node count:" << nodes.size() << std::endl;
    
    // Done!
    return true;
}
```
