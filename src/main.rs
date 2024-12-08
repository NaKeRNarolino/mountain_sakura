use std::fs;
use crate::interpreter::Interpreter;
use crate::parser::Parser;

mod lexer;
mod parser;
mod interpreter;

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

    // dbg!(parser.gen_ast());
    dbg!(interpreter.eval_program());
}
