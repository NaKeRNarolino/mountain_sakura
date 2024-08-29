pub enum Token {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Keyword(KeywordType)
}

pub enum KeywordType {
    FunctionDeclaration,
    ForLoop,
    WhileLoop,
    Let,
    If,
    Else,
    Use
}