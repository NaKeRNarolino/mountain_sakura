use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Keyword(KeywordType),
    Operator(OperatorType),
    Sign(SignType)
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
    Final,
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
        ("final", KeywordType::Final),
        ("const", KeywordType::Const),
        ("typeof", KeywordType::Typeof),
        ("if", KeywordType::If),
        ("else", KeywordType::Else),
        ("enum", KeywordType::Enum),
        ("pri", KeywordType::Pri),
        ("exp", KeywordType::Exp)
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
    ])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignType {
    Semicolon, // ;
    Colon, // :
    Comma, // ,
    Dot, // .
    Underscore, // _
    Arrow, // ->
    BackwardArrow, // <-
    ExclamationMk, // !
    QuestionMk, // ?,
    Paren(Direction), // ( )
    Brace(Direction), // [ ]
    CurlyBrace(Direction), // { }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Direction {
    Open,
    Close
}

pub fn simple_sign_types() -> HashMap<char, SignType> {
    HashMap::from([
        (',', SignType::Comma),
        ('.', SignType::Dot),
        ('_', SignType::Underscore),
        (';', SignType::Semicolon),
        ('?', SignType::QuestionMk),
        ('!', SignType::ExclamationMk)
    ])
}