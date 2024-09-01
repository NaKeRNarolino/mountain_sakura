use std::fs;
use crate::lexer::tokenize;
use crate::parser::Parser;

mod lexer;
mod parser;

fn get_input() -> String {
    let file = fs::read_to_string("./input/main.mosa").unwrap();

    file
}

fn main() {
    let file = get_input();

    let mut parser = Parser::new(file);

    dbg!(parser.gen_ast());
}
