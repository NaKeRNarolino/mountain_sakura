use crate::interpreter::environment::RuntimeScope;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use std::fs;
use std::time::Instant;
use uuid::Uuid;

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

    let scope = RuntimeScope::new(None);
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
