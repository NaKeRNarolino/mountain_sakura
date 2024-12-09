use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Keyword(KeywordType),
    Operator(OperatorType),
    Sign(SignType),
    /// note: should not be used outside lexing process
    Skip,
    End,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum KeywordType {
    Fn,
    For,
    While,
    Use,
    Has,
    Class,
    Let,
    Const,
    Typeof,
    If,
    Else,
    Pri,
    Exp,
    Enum,
}

pub fn reserved_keywords<'a>() -> HashMap<&'a str, KeywordType> {
    HashMap::from([
        ("fn", KeywordType::Fn),
        ("for", KeywordType::For),
        ("while", KeywordType::While),
        ("has", KeywordType::Has),
        ("use", KeywordType::Use),
        ("class", KeywordType::Class),
        ("let", KeywordType::Let),
        ("const", KeywordType::Const),
        ("typeof", KeywordType::Typeof),
        ("if", KeywordType::If),
        ("else", KeywordType::Else),
        ("enum", KeywordType::Enum),
        ("pri", KeywordType::Pri),
        ("exp", KeywordType::Exp),
    ])
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OperatorType {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Increment,
    Decrement,
    Bigger,
    Smaller,
    BiggerEqual,
    SmallerEqual,
    Equal,
    SelfAssign,
    Repeat
}

pub fn simple_operator_types<'a>() -> HashMap<&'a str, OperatorType> {
    HashMap::from([
        ("+", OperatorType::Plus),
        ("-", OperatorType::Minus),
        ("*", OperatorType::Multiply),
        ("/", OperatorType::Divide),
        ("%", OperatorType::Modulo),
        (">", OperatorType::Bigger),
        ("<", OperatorType::Smaller),
        ("=", OperatorType::Equal),
    ])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignType {
    Semicolon,             // ;
    Colon,                 // :
    Comma,                 // ,
    Dot,                   // .
    Underscore,            // _
    Arrow,                 // ->
    BackwardArrow,         // <-
    ExclamationMk,         // !
    QuestionMk,            // ?,
    Paren(Direction),      // ( )
    Brace(Direction),      // [ ]
    CurlyBrace(Direction), // { }
    EqArrow,               // =>
    DoubleArrow,           // ->>
    Comment,               // //
    Equality,              // ==
    Inequality,            // !=
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Direction {
    Open,
    Close,
}

pub fn simple_sign_types() -> HashMap<char, SignType> {
    HashMap::from([
        (',', SignType::Comma),
        ('.', SignType::Dot),
        ('_', SignType::Underscore),
        (';', SignType::Semicolon),
        ('?', SignType::QuestionMk),
        (':', SignType::Colon),
        ('!', SignType::ExclamationMk),
    ])
}

pub struct TwoElementSignsConversion {
    pub first: Token,
    pub second: Token,
    pub result: Token,
}

pub fn two_element_signs_conversions() -> Vec<TwoElementSignsConversion> {
    vec![
        TwoElementSignsConversion {
            first: Token::Operator(OperatorType::Plus),
            second: Token::Operator(OperatorType::Plus),
            result: Token::Operator(OperatorType::Increment),
        }, // ++
        TwoElementSignsConversion {
            first: Token::Operator(OperatorType::Minus),
            second: Token::Operator(OperatorType::Minus),
            result: Token::Operator(OperatorType::Decrement),
        }, // --
        TwoElementSignsConversion {
            first: Token::Operator(OperatorType::Minus),
            second: Token::Operator(OperatorType::Bigger),
            result: Token::Sign(SignType::Arrow),
        }, // ->
        TwoElementSignsConversion {
            first: Token::Operator(OperatorType::Smaller),
            second: Token::Operator(OperatorType::Minus),
            result: Token::Sign(SignType::BackwardArrow),
        }, // <-
        TwoElementSignsConversion {
            first: Token::Operator(OperatorType::Equal),
            second: Token::Operator(OperatorType::Bigger),
            result: Token::Sign(SignType::EqArrow),
        }, // =>
        TwoElementSignsConversion {
            first: Token::Operator(OperatorType::Bigger),
            second: Token::Operator(OperatorType::Equal),
            result: Token::Operator(OperatorType::BiggerEqual),
        }, // >=
        TwoElementSignsConversion {
            first: Token::Operator(OperatorType::Smaller),
            second: Token::Operator(OperatorType::Equal),
            result: Token::Operator(OperatorType::SmallerEqual),
        }, // <=
        TwoElementSignsConversion {
            first: Token::Sign(SignType::Arrow),
            second: Token::Operator(OperatorType::Bigger),
            result: Token::Sign(SignType::DoubleArrow),
        }, // ->>
        TwoElementSignsConversion {
            first: Token::Operator(OperatorType::Equal),
            second: Token::Operator(OperatorType::Equal),
            result: Token::Sign(SignType::Equality),
        }, // ==
        TwoElementSignsConversion {
            first: Token::Sign(SignType::ExclamationMk),
            second: Token::Operator(OperatorType::Equal),
            result: Token::Sign(SignType::Inequality),
        }, // !=
        TwoElementSignsConversion {
            first: Token::Operator(OperatorType::Divide),
            second: Token::Operator(OperatorType::Divide),
            result: Token::Sign(SignType::Comment),
        }, // //
        TwoElementSignsConversion {
            first: Token::Sign(SignType::Colon),
            second: Token::Operator(OperatorType::Equal),
            result: Token::Operator(OperatorType::SelfAssign),
        }, // :=
        TwoElementSignsConversion {
            first: Token::Sign(SignType::QuestionMk),
            second: Token::Sign(SignType::Colon),
            result: Token::Operator(OperatorType::Repeat),
        }, // ?:
    ]
}
