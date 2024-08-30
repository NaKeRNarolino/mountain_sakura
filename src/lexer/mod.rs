use std::any::{Any};
use std::cmp::PartialEq;
use std::collections::VecDeque;

// use regex::RegexBuilder;
use crate::lexer::structs::{reserved_keywords, simple_operator_types, simple_sign_types, two_element_signs_conversions, Direction, SignType, Token};
pub mod structs;

fn is_skippable(input: char) -> bool {
    input == ' ' || input == '\n' || input == '\r'
}

fn resolve_string_to_token(input: String) -> Token {
    // let regex = RegexBuilder::new("^[A-Za-z][A-Za-z0-9]*$").build().unwrap();

    if let Some(token) = reserved_keywords().get(input.as_str()).cloned() {
        Token::Keyword(token)
    }
    else {
        Token::Identifier(input)
    }
}

pub fn tokenize(input: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let mut input_chars: VecDeque<char> = input.chars().collect();


    while !input_chars.is_empty() {
        if let Some(char) = input_chars.pop_front() {
            if is_skippable(char) {
                tokens.push(Token::Skip);
                continue;
            }
            // PARENS
            if char == '(' {
                tokens.push(Token::Sign(SignType::Paren(Direction::Open)))
            } else if char == ')' {
                tokens.push(Token::Sign(SignType::Paren(Direction::Close)))
            }
            // Braces
            else if char == '[' {
                tokens.push(Token::Sign(SignType::Brace(Direction::Open)))
            } else if char == ']' {
                tokens.push(Token::Sign(SignType::Brace(Direction::Close)))
            }
            // Curly-s
            else if char == '{' {
                tokens.push(Token::Sign(SignType::CurlyBrace(Direction::Open)))
            } else if char == '}' {
                tokens.push(Token::Sign(SignType::CurlyBrace(Direction::Close)))
            } else if let Some(sign_type) = simple_sign_types().get(&char).cloned() {
                tokens.push(Token::Sign(sign_type));
            } else if let Some(op) = simple_operator_types().get(format!("{}", char).as_str()).cloned() {
                if let Some(last_el) = tokens.last().cloned() {
                    let mut found = false;

                    for conversion in two_element_signs_conversions() {
                        if conversion.first == last_el && conversion.second == Token::Operator(op.clone()) {
                            found = true;
                            tokens.pop();
                            tokens.push(conversion.result);
                        }
                    }

                    if !found {
                        tokens.push(Token::Operator(op));
                    }
                }
                else {
                    tokens.push(Token::Operator(op));
                }
            }
            else {
                // STRINGS OR NUMBERS
                if char.is_alphabetic() {
                    let mut identifier_string = String::new();

                    identifier_string.push(char);

                    dbg!(input_chars[0]);

                    if input_chars.is_empty() || !input_chars[0].is_alphabetic() || is_skippable(input_chars[0]) {
                        if identifier_string.chars().last().unwrap_or('!') == '\r' {
                            identifier_string.pop();
                        }

                        tokens.push(resolve_string_to_token(identifier_string));

                        // input_chars.push_back(iter_char);
                        continue;
                    }
                    let mut iter_char: char = input_chars.pop_front().unwrap_or('\r');

                    loop {
                        identifier_string.push(iter_char);

                        if input_chars.is_empty() || !input_chars[0].is_alphabetic() || is_skippable(input_chars[0]) {
                            if identifier_string.chars().last().unwrap_or('!') == '\r' {
                                identifier_string.pop();
                            }

                            tokens.push(resolve_string_to_token(identifier_string));

                            // input_chars.push_back(iter_char);

                            break;
                        }

                        iter_char = input_chars.pop_front().unwrap_or('\r');
                    }
                }
            }
        }
    }

    tokens
}