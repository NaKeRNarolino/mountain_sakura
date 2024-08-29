use std::collections::VecDeque;
use crate::lexer::structs::Token;

pub mod structs;

fn is_skippable(input: char) -> bool {
    input == ' ' || input == '\n'
}

pub fn tokenize(input: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let mut input_chars: VecDeque<char> = input.chars().collect();


    let mut last_char = ' ';
    while !input_chars.is_empty() {
        if let Some(char) = input_chars.pop_front() {
            if is_skippable(char) {
                last_char = char;
                continue;
            }

            if char.is_digit(10) && is_skippable(last_char) {
                let mut number_string = String::new();

                number_string.push(char);

                let mut num_char;
                loop {
                    num_char = input_chars.pop_front().unwrap();

                    if !num_char.is_digit(10) && num_char != '.' {
                        break;
                    }

                    number_string.push(num_char);
                }


                if !number_string.contains('.') {
                    number_string.push('.');
                    number_string.push('0');
                }


                tokens.push(Token::Number(
                    number_string.parse::<f64>().unwrap()
                ))
            }

            last_char = char;
        }
    }

    tokens
}