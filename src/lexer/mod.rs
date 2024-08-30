use std::any::{Any, TypeId};
use std::cmp::PartialEq;
use std::collections::VecDeque;
use std::fmt::format;
use std::mem::swap;
use regex::RegexBuilder;
use crate::lexer::structs::{reserved_keywords, simple_operator_types, simple_sign_types, Direction, OperatorType, SignType, Token};
pub mod structs;

fn is_skippable(input: char) -> bool {
    input == ' ' || input == '\n' || input == '\r'
}

fn resolve_string_to_token(input: String) -> Token {
    let regex = RegexBuilder::new("^[A-Za-z][A-Za-z0-9]*$").build().unwrap();

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


    let mut last_char = ' ';
    while !input_chars.is_empty() {
        if let Some(char) = input_chars.pop_front() {
            if is_skippable(char) {
                continue;
            }
            // PARENS
            if (char == '(') {
                tokens.push(Token::Sign(SignType::Paren(Direction::Open)))
            } else if (char == ')') {
                tokens.push(Token::Sign(SignType::Paren(Direction::Close)))
            }
            // Braces
            else if (char == '[') {
                tokens.push(Token::Sign(SignType::Brace(Direction::Open)))
            } else if (char == ']') {
                tokens.push(Token::Sign(SignType::Brace(Direction::Close)))
            }
            // Curly-s
            else if (char == '{') {
                tokens.push(Token::Sign(SignType::CurlyBrace(Direction::Open)))
            } else if (char == '}') {
                tokens.push(Token::Sign(SignType::CurlyBrace(Direction::Close)))
            } else if let Some(sign_type) = simple_sign_types().get(&char).cloned() {
                tokens.push(Token::Sign(sign_type));
            } else if let Some(op) = simple_operator_types().get(format!("{}", char).as_str()).cloned() {
                if tokens.last().is_some() && tokens.last().unwrap().clone() == Token::Operator(OperatorType::Plus) && op == OperatorType::Plus {
                    tokens.pop();
                    tokens.push(Token::Operator(OperatorType::Increment));
                }
                else if tokens.last().is_some() && tokens.last().unwrap().clone() == Token::Operator(OperatorType::Minus) && op == OperatorType::Bigger {
                    tokens.pop();
                    tokens.push(Token::Sign(SignType::Arrow));
                }
                else if tokens.last().is_some() && tokens.last().unwrap().clone() == Token::Operator(OperatorType::Bigger) && op == OperatorType::Minus {
                    tokens.pop();
                    tokens.push(Token::Sign(SignType::BackwardArrow));
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

                    let mut iter_char: char = input_chars.pop_front().unwrap_or('\r');

                    loop {
                        identifier_string.push(iter_char);

                        if input_chars.is_empty() || !input_chars[0].is_alphabetic() || is_skippable(input_chars[0]) {
                            if (identifier_string.chars().last().unwrap_or('!') == '\r') {
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