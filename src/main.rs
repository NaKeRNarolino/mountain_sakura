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

pub mod global;
pub mod interpreter;
pub mod lexer;
pub mod parser;

fn get_input() -> String {
    let file = fs::read_to_string("./input/main.mosa").unwrap();

    file
}

fn main() {
    let file = get_input();

    let mut parser = Parser::new(file);
    let ast = parser.gen_ast();
    let interpreter = Interpreter::new(ast.clone());

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
