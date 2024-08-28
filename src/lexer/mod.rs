use regex::{Regex, RegexBuilder};
use crate::lexer::structs::Token;

pub mod structs;


fn is_numeric(input: &str) -> bool {
    let regex = RegexBuilder::new(r"^-?\d+(\.\d+)?$").build().unwrap();

    regex.is_match(input)
}



pub fn tokenize(input: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];



    tokens
}