use std::collections::VecDeque;

use crate::lexer::structs::{
    reserved_keywords, simple_operator_types, simple_sign_types, two_element_signs_conversions,
    Direction, SignType, Token, TokenValue,
};
use regex::RegexBuilder;
pub mod structs;

fn is_skippable(input: char) -> bool {
    input == ' ' || input == '\n' || input == '\r'
}

fn resolve_string_to_token(input: String) -> TokenValue {
    // let regex = RegexBuilder::new("^[A-Za-z][A-Za-z0-9]*$").build().unwrap();

    if let Some(token) = reserved_keywords().get(input.as_str()).cloned() {
        TokenValue::Keyword(token)
    } else if input == String::from("true") || input == String::from("false") {
        TokenValue::Boolean(if input == String::from("true") {
            true
        } else {
            false
        })
    } else {
        TokenValue::Identifier(input)
    }
}

pub fn tokenize(file_name: String, raw_input: String) -> VecDeque<Token> {
    let mut tokens: Vec<Token> = vec![];

    let input = RegexBuilder::new(r"(\/\/).*")
        .build()
        .unwrap()
        .replace_all(&raw_input, "")
        .to_string();

    let mut input_chars: VecDeque<char> = input.chars().collect();
    let mut making_string: bool = false;
    let mut string = String::new();
    let mut prev_char: char = '\r';
    let mut line = 1;
    let mut column = 0;

    while !input_chars.is_empty() {
        if let Some(char) = input_chars.pop_front() {
            column += 1;
            if char == '\n' {
                column = 0;
                line += 1;
            }
            if char == '"' && prev_char != '\\' {
                prev_char = char;
                making_string = !making_string;
                if making_string == false {
                    tokens.push(Token {
                        value: TokenValue::String(string.clone().replace("\\\"", "\"")),
                        file_name: file_name.clone(),
                        line,
                        column,
                    });
                    string = String::from("");
                }
                continue;
            }
            prev_char = char;

            if making_string {
                dbg!(&char);
                string.push(char);
                continue;
            }

            if is_skippable(char) {
                tokens.push(Token {
                    value: TokenValue::Skip,
                    line,
                    column,
                    file_name: file_name.clone(),
                });
                continue;
            }
            // PARENS
            if char == '(' {
                tokens.push(Token {
                    value: TokenValue::Sign(SignType::Paren(Direction::Open)),
                    line,
                    column,
                    file_name: file_name.clone(),
                });
            } else if char == ')' {
                tokens.push(Token {
                    value: TokenValue::Sign(SignType::Paren(Direction::Close)),
                    line,
                    column,
                    file_name: file_name.clone(),
                })
            }
            // Braces
            else if char == '[' {
                tokens.push(Token {
                    value: TokenValue::Sign(SignType::Brace(Direction::Open)),
                    line,
                    column,
                    file_name: file_name.clone(),
                })
            } else if char == ']' {
                tokens.push(Token {
                    value: TokenValue::Sign(SignType::Brace(Direction::Close)),
                    line,
                    column,
                    file_name: file_name.clone(),
                })
            }
            // Curly-s
            else if char == '{' {
                tokens.push(Token {
                    value: TokenValue::Sign(SignType::CurlyBrace(Direction::Open)),
                    line,
                    column,
                    file_name: file_name.clone(),
                })
            } else if char == '}' {
                tokens.push(Token {
                    value: TokenValue::Sign(SignType::CurlyBrace(Direction::Close)),
                    line,
                    column,
                    file_name: file_name.clone(),
                })
            } else if let Some(sign_type) = simple_sign_types().get(&char).cloned() {
                if let Some(last_el) = tokens.last().cloned() {
                    let mut found = false;

                    for conversion in two_element_signs_conversions() {
                        if conversion.first == last_el.value
                            && conversion.second == TokenValue::Sign(sign_type.clone())
                        {
                            found = true;
                            tokens.pop();
                            tokens.push(Token {
                                value: conversion.result,
                                line,
                                column,
                                file_name: file_name.clone(),
                            });
                        }
                    }

                    if !found {
                        tokens.push(Token {
                            value: TokenValue::Sign(sign_type),
                            line,
                            column,
                            file_name: file_name.clone(),
                        });
                    }
                } else {
                    tokens.push(Token {
                        value: TokenValue::Sign(sign_type),
                        line,
                        column,
                        file_name: file_name.clone(),
                    });
                }
            } else if let Some(op) = simple_operator_types()
                .get(format!("{}", char).as_str())
                .cloned()
            {
                if let Some(last_el) = tokens.last().cloned() {
                    let mut found = false;

                    for conversion in two_element_signs_conversions() {
                        if conversion.first == last_el.value
                            && conversion.second == TokenValue::Operator(op.clone())
                        {
                            found = true;
                            tokens.pop();
                            tokens.push(Token {
                                value: conversion.result,
                                line,
                                column,
                                file_name: file_name.clone(),
                            });
                        }
                    }

                    if !found {
                        tokens.push(Token {
                            value: TokenValue::Operator(op),
                            line,
                            column,
                            file_name: file_name.clone(),
                        });
                    }
                } else {
                    tokens.push(Token {
                        value: TokenValue::Operator(op),
                        line,
                        column,
                        file_name: file_name.clone(),
                    });
                }
            } else {
                // STRINGS OR NUMBERS
                if char.is_alphabetic() && !char.is_numeric() || input_chars[0] == '_' {
                    let mut identifier_string = String::new();

                    identifier_string.push(char);

                    if input_chars.is_empty()
                        || (!input_chars[0].is_alphabetic()
                            && !input_chars[0].is_numeric()
                            && input_chars[0] != '_')
                        || is_skippable(input_chars[0])
                    {
                        if identifier_string.chars().last().unwrap_or('!') == '\r' {
                            identifier_string.pop();
                        }

                        tokens.push(Token {
                            value: resolve_string_to_token(identifier_string),
                            line,
                            column,
                            file_name: file_name.clone(),
                        });

                        // input_chars.push_back(iter_char);
                        continue;
                    }
                    let mut iter_char: char = input_chars.pop_front().unwrap_or('\r');

                    loop {
                        identifier_string.push(iter_char);

                        if (input_chars.is_empty()
                            || (!input_chars[0].is_alphabetic() && !input_chars[0].is_numeric())
                            || is_skippable(input_chars[0]))
                            && input_chars.get(0).cloned().unwrap_or(' ') != '_'
                        {
                            if identifier_string.chars().last().unwrap_or('!') == '\r' {
                                identifier_string.pop();
                            }

                            tokens.push(Token {
                                value: resolve_string_to_token(identifier_string),
                                line,
                                column,
                                file_name: file_name.clone(),
                            });

                            // input_chars.push_back(iter_char);

                            break;
                        }

                        iter_char = input_chars.pop_front().unwrap_or('\r');
                    }
                } else if char.is_numeric() {
                    let mut number_str = String::new();

                    number_str.push(char);

                    if input_chars.is_empty()
                        || (input_chars[0] == '.'
                            && input_chars.get(1).unwrap_or(&'.').clone() == '.')
                        || (!input_chars[0].is_numeric() && input_chars[0] != '.')
                        || is_skippable(input_chars[0])
                    {
                        if number_str.chars().last().unwrap_or('!') == '\r' {
                            number_str.pop();
                        }

                        tokens.push(Token {
                            value: TokenValue::Number(number_str.parse::<f64>().unwrap()),
                            line,
                            column,
                            file_name: file_name.clone(),
                        });

                        // input_chars.push_back(iter_char);
                        continue;
                    }
                    let mut iter_char: char = input_chars.pop_front().unwrap_or('\r');

                    loop {
                        number_str.push(iter_char);

                        if input_chars.is_empty()
                            || (!input_chars[0].is_numeric() && input_chars[0] != '.')
                            || (input_chars[0] == '.'
                                && input_chars.get(1).unwrap_or(&'.').clone() == '.')
                            || is_skippable(input_chars[0])
                        {
                            if number_str.chars().last().unwrap_or('!') == '\r' {
                                number_str.pop();
                            }

                            if !number_str.contains('.') {
                                number_str.push('.');
                                number_str.push('0');
                            }

                            dbg!(&number_str);

                            tokens.push(Token {
                                value: TokenValue::Number(number_str.parse::<f64>().unwrap()),
                                line,
                                column,
                                file_name: file_name.clone(),
                            });

                            // input_chars.push_back(iter_char);

                            break;
                        }

                        iter_char = input_chars.pop_front().unwrap_or('\r');
                    }
                }
            }
        }
    }

    tokens = tokens
        .into_iter()
        .filter(|x| &x.value != &TokenValue::Skip)
        .collect();

    tokens.push(Token {
        value: TokenValue::End,
        line,
        column,
        file_name: file_name.clone(),
    });

    tokens.into()
}
