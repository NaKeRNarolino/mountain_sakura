use std::fs;
use crate::lexer::structs::Token;
use crate::lexer::tokenize;

mod lexer;

fn get_input() -> String {
    let file = fs::read_to_string("./input/main.mosa").unwrap();

    file
}

fn main() {
    let file = get_input();
    let tokens = tokenize(file);

    for token in tokens {
        if let Token::Number(num) = token {
            println!("{}", num);
        }
    }
}
