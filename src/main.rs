use crate::interpreter::scope::RuntimeScope;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use std::fs;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use crate::global::DataType;
use crate::global::PrimitiveDataType;
use crate::interpreter::structs::RuntimeValue;
use crate::modules::{Module, ModuleStorage};

pub mod global;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod modules;
pub mod mosa_fs;
pub mod logging;


fn get_input() -> String {
    let file = fs::read_to_string("./input/main.mosa").unwrap();

    file
}

fn main() {
    let file = get_input();
    
    let module_storage = Arc::new(ModuleStorage::new());
    let module = Module::new("main".to_string());
    
    let mut parser = Parser::new(file, module, module_storage.clone(), "/home/nakernarolino/RustroverProjects/mountain_sakura/input/".to_string(), "".to_string());
    let ast = parser.gen_ast();
    let interpreter = Interpreter::new(ast.clone(), module_storage);
    
    dbg!(ast);
    
    let mut scope = RuntimeScope::new(None);
    
    scope.declare_variable("null".to_string(), DataType::Primitive(PrimitiveDataType::Null), RuntimeValue::Null, true);
    
    scope.add_native_function(String::from("mosa-native~>printLn"), Arc::new(|args| {
        println!("{}", args[0]);
        std::io::stdout().flush().unwrap();
    
        RuntimeValue::Null
    }));
    
    scope.add_native_function(String::from("mosa-native~>print"), Arc::new(|args| {
        print!("{}", args[0]);
        RuntimeValue::Null
    }));
    

    let time = Instant::now();
    //
    dbg!(interpreter.eval_program(scope));
    //
    let elapsed = time.elapsed();
    //
    dbg!(elapsed);

    // logging::error("error message".to_string(), 10, 12, "main".to_string());
}
