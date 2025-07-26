use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Token {
    pub file_name: String,
    pub line: usize,
    pub column: usize,
    pub value: TokenValue
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
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
    Immut,
    Once,
    Native,
    Block,
    Layout,
    Mix,
    Tied
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
        ("immut", KeywordType::Immut),
        ("once", KeywordType::Once),
        ("native", KeywordType::Native),
        ("block", KeywordType::Block),
        ("layout", KeywordType::Layout),
        ("mix", KeywordType::Mix),
        ("tied", KeywordType::Tied)
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
    Repeat,
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
    QuestionMk,            // ?
    Paren(Direction),      // ( )
    Brace(Direction),      // [ ]
    CurlyBrace(Direction), // { }
    EqArrow,               // =>
    DoubleArrow,           // ->>
    Comment,               // //
    Equality,              // ==
    Inequality,            // !=
    HashSign,              // #
    Caret,                 // ^
    DoubleDot,             // ..
    SlashArrow,            // />
    At,                    // @
    Tilde,                 // ~
    TildeArrow,            // ~>
    DoubleColon,           // ::
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
        ('#', SignType::HashSign),
        ('^', SignType::Caret),
        ('@', SignType::At),
        ('~', SignType::Tilde)
    ])
}

pub struct TwoElementSignsConversion {
    pub first: TokenValue,
    pub second: TokenValue,
    pub result: TokenValue,
}

pub fn two_element_signs_conversions() -> Vec<TwoElementSignsConversion> {
    vec![
        TwoElementSignsConversion {
            first: TokenValue::Operator(OperatorType::Plus),
            second: TokenValue::Operator(OperatorType::Plus),
            result: TokenValue::Operator(OperatorType::Increment),
        }, // ++
        TwoElementSignsConversion {
            first: TokenValue::Operator(OperatorType::Minus),
            second: TokenValue::Operator(OperatorType::Minus),
            result: TokenValue::Operator(OperatorType::Decrement),
        }, // --
        TwoElementSignsConversion {
            first: TokenValue::Operator(OperatorType::Minus),
            second: TokenValue::Operator(OperatorType::Bigger),
            result: TokenValue::Sign(SignType::Arrow),
        }, // ->
        TwoElementSignsConversion {
            first: TokenValue::Operator(OperatorType::Smaller),
            second: TokenValue::Operator(OperatorType::Minus),
            result: TokenValue::Sign(SignType::BackwardArrow),
        }, // <-
        TwoElementSignsConversion {
            first: TokenValue::Operator(OperatorType::Equal),
            second: TokenValue::Operator(OperatorType::Bigger),
            result: TokenValue::Sign(SignType::EqArrow),
        }, // =>
        TwoElementSignsConversion {
            first: TokenValue::Operator(OperatorType::Bigger),
            second: TokenValue::Operator(OperatorType::Equal),
            result: TokenValue::Operator(OperatorType::BiggerEqual),
        }, // >=
        TwoElementSignsConversion {
            first: TokenValue::Operator(OperatorType::Smaller),
            second: TokenValue::Operator(OperatorType::Equal),
            result: TokenValue::Operator(OperatorType::SmallerEqual),
        }, // <=
        TwoElementSignsConversion {
            first: TokenValue::Sign(SignType::Arrow),
            second: TokenValue::Operator(OperatorType::Bigger),
            result: TokenValue::Sign(SignType::DoubleArrow),
        }, // ->>
        TwoElementSignsConversion {
            first: TokenValue::Operator(OperatorType::Equal),
            second: TokenValue::Operator(OperatorType::Equal),
            result: TokenValue::Sign(SignType::Equality),
        }, // ==
        TwoElementSignsConversion {
            first: TokenValue::Sign(SignType::ExclamationMk),
            second: TokenValue::Operator(OperatorType::Equal),
            result: TokenValue::Sign(SignType::Inequality),
        }, // !=
        TwoElementSignsConversion {
            first: TokenValue::Operator(OperatorType::Divide),
            second: TokenValue::Operator(OperatorType::Divide),
            result: TokenValue::Sign(SignType::Comment),
        }, // //
        TwoElementSignsConversion {
            first: TokenValue::Sign(SignType::Colon),
            second: TokenValue::Operator(OperatorType::Equal),
            result: TokenValue::Operator(OperatorType::SelfAssign),
        }, // :=
        TwoElementSignsConversion {
            first: TokenValue::Sign(SignType::QuestionMk),
            second: TokenValue::Sign(SignType::Colon),
            result: TokenValue::Operator(OperatorType::Repeat),
        }, // ?:
        TwoElementSignsConversion {
            first: TokenValue::Sign(SignType::Dot),
            second: TokenValue::Sign(SignType::Dot),
            result: TokenValue::Sign(SignType::DoubleDot),
        }, // ..
        TwoElementSignsConversion {
            first: TokenValue::Operator(OperatorType::Divide),
            second: TokenValue::Operator(OperatorType::Bigger),
            result: TokenValue::Sign(SignType::SlashArrow),
        }, // />
        TwoElementSignsConversion {
            first: TokenValue::Sign(SignType::Tilde),
            second: TokenValue::Operator(OperatorType::Bigger),
            result: TokenValue::Sign(SignType::TildeArrow),
        }, // ~>
        TwoElementSignsConversion {
            first: TokenValue::Sign(SignType::Colon),
            second: TokenValue::Sign(SignType::Colon),
            result: TokenValue::Sign(SignType::DoubleColon),
        }, // ::
    ]
}
