use std::fs;
use std::io::Write;
use std::string::String;
use std::sync::Arc;
use std::time::Instant;
use ::jni::{JNIEnv};
use ::jni::objects::{JClass, JObject, JObjectArray, JString};
use crate::global::{DataType, PrimitiveDataType};
use crate::interpreter::Interpreter;
use crate::interpreter::scope::RuntimeScope;
use crate::interpreter::structs::RuntimeValue;
use crate::jni::jni::{assign_global_jvm};
use crate::modules::{Module, ModuleStorage};
use crate::opts::{parseopts, OptValue};
use crate::parser::Parser;

pub mod global;
pub mod interpreter;
pub mod jni;
pub mod lexer;
pub mod logging;
pub mod modules;
pub mod mosa_fs;

pub mod jni;
pub mod opts;
pub mod parser;
pub mod wrapper;
pub mod prelude;


#[no_mangle]
pub extern "system" fn Java_dev_kofeychi_mosajni_MosaJniBinds_jni<'local>(env: JNIEnv<'local>,
                                                                  _class: JClass<'local>) {
    assign_global_jvm(env);
    println!("JVM assigned")
}

fn get_input(path: String) -> String {
    let file = fs::read_to_string(path).unwrap();

    file
}
fn parse_java_string_array(
    env: &mut JNIEnv,
    array: JObjectArray,
) -> Vec<String> {
    let len = env.get_array_length(&array).unwrap();
    let mut vec = vec!();

    for i in 0..len {
        let element = env.get_object_array_element(&array, i).unwrap();
        let jstring = JString::from(element);
        let rust_str = env.get_string(&jstring).unwrap();
        vec.push(rust_str.to_string_lossy().into_owned());
    }
    vec
}
#[no_mangle]
pub extern "system" fn Java_dev_kofeychi_mosajni_MosaJniBinds_eval<'local>(mut env: JNIEnv<'local>,
                                                                          class: JClass<'local>,
                                                                           args: JObjectArray<'local>
) {
    let arr: Vec<String> = parse_java_string_array(&mut env, args);
    dbg!(&arr);

    let opts = parseopts(arr);
    let workdir = match opts.get("workdir").unwrap() {
        OptValue::String(a) => {a}
        _ => {""}
    };
    let path = match opts.get("main").unwrap() {
        OptValue::String(a) => {a}
        _ => {""}
    };

    let file = get_input(path.to_string());

    let module_storage = Arc::new(ModuleStorage::new());
    let module = Module::new("main".to_string());

    let mut parser = Parser::new(file, module, module_storage.clone(), workdir.to_string(), "".to_string());
    let ast = parser.gen_ast();
    let interpreter = Interpreter::new(ast.clone(), module_storage);

    dbg!(ast);

    let mut scope = RuntimeScope::new(None);

    scope.declare_variable("null".to_string(), DataType::Primitive(PrimitiveDataType::Null), RuntimeValue::Null, true);

    scope.add_native_function(std::string::String::from("mosa-native~>printLn"), Arc::new(|args| {
        println!("{}", args[0]);
        std::io::stdout().flush().unwrap();

        RuntimeValue::Null
    }));

    scope.add_native_function(std::string::String::from("mosa-native~>print"), Arc::new(|args| {
        print!("{}", args[0]);
        RuntimeValue::Null
    }));

    //
    // env_map.insert(Uuid::new_v4(), Environment::new());

    // env.declare_variable(true, String::from("true"), RuntimeValue::Bool(true)).unwrap();
    // env.declare_variable(true, String::from("false"), RuntimeValue::Bool(false)).unwrap();

    // dbg!(parser.gen_ast());

    let time = Instant::now();

    dbg!(interpreter.eval_program(scope));

    let elapsed = time.elapsed();

    dbg!(elapsed);
}