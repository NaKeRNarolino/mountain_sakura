#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeValue {
    Number(f64),
    Null,
    String(String),
    Bool(bool),
    Complex
}