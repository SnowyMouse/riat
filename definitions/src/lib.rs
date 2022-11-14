extern crate proc_macro;
use proc_macro::TokenStream;

extern crate serde;
extern crate serde_json;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Global {
    name: String,
    r#type: String,
    engines: BTreeMap<String, Value>
}

fn default_value<T: Default>() -> T {
    T::default()
}

#[derive(Deserialize)]
struct FunctionParameter {
    r#type: String,

    #[serde(default = "default_value")]
    many: bool,

    #[serde(default = "default_value")]
    allow_uppercase: bool,

    #[serde(default = "default_value")]
    optional: bool
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct Function {
    name: String,

    #[serde(default = "default_value")]
    description: String,

    r#type: String,

    #[serde(default = "default_value")]
    parameters: Vec<FunctionParameter>,

    #[serde(default = "default_value")]
    number_passthrough: bool,

    #[serde(default = "default_value")]
    inequality: bool,

    #[serde(default = "default_value")]
    passthrough_last: bool,

    engines: BTreeMap<String, Value>
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DefinitionStruct {
    description: String,
    date: String,
    engines: Vec<Value>,
    functions: Vec<Function>,
    globals: Vec<Global>
}

#[proc_macro]
pub fn generate_definitions(_: TokenStream) -> TokenStream {
    let json = include_bytes!("definition/definition.json");
    let json_slice = &json[..];
    let definitions : DefinitionStruct = serde_json::from_slice(json_slice).unwrap();

    // Convert from snake_case to PascalCase
    fn snake_to_pascal(t: &str) -> String {
        let mut s : Vec<char> = t.chars().collect();
        s[0].make_ascii_uppercase();

        loop {
            if let Some(n) = s.iter().position(|x| { *x == '_' }) {
                s[n + 1].make_ascii_uppercase();
                s.remove(n);
            }
            else {
                break
            }
        }

        format!("ValueType::{}", s.into_iter().collect::<String>())
    }

    // Make a Availability struct
    fn generate_availability(t: &BTreeMap<String, Value>) -> String {
        let mut s = String::new();

        let mut modify_thing = |from: &str, to: &str| {
            if let Some(n) = t.get(from) {
                match n {
                    Value::Null => s += &format!("{to}: Some(u16::MAX),"),
                    Value::Number(n) => {
                        // Indices must be <= 65535
                        let v = n.as_u64().unwrap();
                        assert!(v <= u16::MAX as u64);

                        // Here we go
                        s += &format!("{to}: Some({v}),")
                    },
                    _ => unreachable!()
                }
            }
            else {
                s += &format!("{to}: None,");
            }
        };

        modify_thing("mcc-cea", "mcc_cea");
        modify_thing("xbox", "xbox");
        modify_thing("gbx-custom", "gbx_custom");
        modify_thing("gbx-retail", "gbx_retail");
        modify_thing("gbx-demo", "gbx_demo");

        format!("EngineAvailability {{ {s} }}")
    }

    // Generate globals
    let mut globals_list = String::new();
    for g in &definitions.globals {
        let global_name = &g.name;
        let global_type = snake_to_pascal(&g.r#type);
        let global_availability = generate_availability(&g.engines);

        globals_list += &format!("EngineGlobal {{ name: \"{global_name}\", value_type: {global_type}, availability: {global_availability} }},");
    }

    // Generate functions
    let mut functions_list = String::new();
    for f in &definitions.functions {
        let function_name = &f.name;
        let function_type = snake_to_pascal(&f.r#type);
        let function_availability = generate_availability(&f.engines);
        let function_number_passthrough = &f.number_passthrough;
        let function_passthrough_last = &f.passthrough_last;
        let function_inequality = &f.inequality;

        let mut function_parameters = String::new();
        for p in &f.parameters {
            let parameter_type = snake_to_pascal(&p.r#type);
            let parameter_many = &p.many;
            let parameter_allow_uppercase = &p.allow_uppercase;
            let parameter_optional = &p.optional;
            function_parameters += &format!("EngineFunctionParameter {{ value_type: {parameter_type}, many: {parameter_many}, allow_uppercase: {parameter_allow_uppercase}, optional: {parameter_optional} }},")
        }

        functions_list += &format!("EngineFunction {{ name: \"{function_name}\", return_type: {function_type}, availability: {function_availability}, number_passthrough: {function_number_passthrough}, inequality: {function_inequality}, passthrough_last: {function_passthrough_last}, parameters: &[{function_parameters}] }},");
    }

    format!("pub(crate) const ALL_GLOBALS: [EngineGlobal; {}] = [{}]; pub(crate) const ALL_FUNCTIONS: [EngineFunction; {}] = [{}];", definitions.globals.len(), globals_list, definitions.functions.len(), functions_list).parse().unwrap()
}
