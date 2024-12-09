use crate::interpreter::environment::Environment;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use std::fs;

mod interpreter;
mod lexer;
mod parser;

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

    let mut env = Environment::new();

    // env.declare_variable(true, String::from("true"), RuntimeValue::Bool(true)).unwrap();
    // env.declare_variable(true, String::from("false"), RuntimeValue::Bool(false)).unwrap();

    // dbg!(parser.gen_ast());
    dbg!(interpreter.eval_program(&mut env));
}
