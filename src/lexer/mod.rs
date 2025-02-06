use std::collections::VecDeque;

// use regex::RegexBuilder;
use crate::lexer::structs::{
    reserved_keywords, simple_operator_types, simple_sign_types, two_element_signs_conversions,
    Direction, SignType, Token,
};
pub mod structs;

fn is_skippable(input: char) -> bool {
    input == ' ' || input == '\n' || input == '\r'
}

fn resolve_string_to_token(input: String) -> Token {
    // let regex = RegexBuilder::new("^[A-Za-z][A-Za-z0-9]*$").build().unwrap();

    if let Some(token) = reserved_keywords().get(input.as_str()).cloned() {
        Token::Keyword(token)
    } else if input == String::from("true") || input == String::from("false") {
        Token::Boolean(if input == String::from("true") {
            true
        } else {
            false
        })
    } else {
        Token::Identifier(input)
    }
}

pub fn tokenize(input: String) -> VecDeque<Token> {
    let mut tokens: Vec<Token> = vec![];

    let mut input_chars: VecDeque<char> = input.chars().collect();
    let mut making_string: bool = false;
    let mut string = String::new();
    let mut prev_char: char = '\r';

    while !input_chars.is_empty() {
        if let Some(char) = input_chars.pop_front() {
            if char == '"' && prev_char != '\\' {
                making_string = !making_string;
                if making_string == false {
                    tokens.push(Token::String(string.clone().replace("\\\"", "\"")));
                    string = String::from("");
                }
                prev_char = char;
                continue;
            }
            prev_char = char;

            if making_string {
                string.push(char);
                continue;
            }

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
                if let Some(last_el) = tokens.last().cloned() {
                    let mut found = false;

                    for conversion in two_element_signs_conversions() {
                        if conversion.first == last_el
                            && conversion.second == Token::Sign(sign_type.clone())
                        {
                            found = true;
                            tokens.pop();
                            tokens.push(conversion.result);
                        }
                    }

                    if !found {
                        tokens.push(Token::Sign(sign_type));
                    }
                } else {
                    tokens.push(Token::Sign(sign_type));
                }
            } else if let Some(op) = simple_operator_types()
                .get(format!("{}", char).as_str())
                .cloned()
            {
                if let Some(last_el) = tokens.last().cloned() {
                    let mut found = false;

                    for conversion in two_element_signs_conversions() {
                        if conversion.first == last_el
                            && conversion.second == Token::Operator(op.clone())
                        {
                            found = true;
                            tokens.pop();
                            tokens.push(conversion.result);
                        }
                    }

                    if !found {
                        tokens.push(Token::Operator(op));
                    }
                } else {
                    tokens.push(Token::Operator(op));
                }
            } else {
                // STRINGS OR NUMBERS
                if char.is_alphabetic() && !char.is_numeric() {
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

                        tokens.push(resolve_string_to_token(identifier_string));

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

                            tokens.push(resolve_string_to_token(identifier_string));

                            // input_chars.push_back(iter_char);

                            break;
                        }

                        iter_char = input_chars.pop_front().unwrap_or('\r');
                    }
                } else if char.is_numeric() {
                    let mut number_str = String::new();

                    number_str.push(char);

                    if input_chars.is_empty()
                        || (!input_chars[0].is_numeric() && input_chars[0] != '.')
                        || is_skippable(input_chars[0])
                    {
                        if number_str.chars().last().unwrap_or('!') == '\r' {
                            number_str.pop();
                        }

                        tokens.push(Token::Number(number_str.parse::<f64>().unwrap()));

                        // input_chars.push_back(iter_char);
                        continue;
                    }
                    let mut iter_char: char = input_chars.pop_front().unwrap_or('\r');

                    loop {
                        number_str.push(iter_char);

                        if input_chars.is_empty()
                            || (!input_chars[0].is_numeric() && input_chars[0] != '.')
                            || is_skippable(input_chars[0])
                        {
                            if number_str.chars().last().unwrap_or('!') == '\r' {
                                number_str.pop();
                            }

                            if !number_str.contains('.') {
                                number_str.push('.');
                                number_str.push('0');
                            }

                            tokens.push(Token::Number(number_str.parse::<f64>().unwrap()));

                            // input_chars.push_back(iter_char);

                            break;
                        }

                        iter_char = input_chars.pop_front().unwrap_or('\r');
                    }
                }
            }
        }
    }

    tokens = tokens.into_iter().filter(|x| x != &Token::Skip).collect();

    tokens.push(Token::End);

    tokens.into()
}
