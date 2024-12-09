use std::ops::{Add, Sub};

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(f64),
    Null,
    String(String),
    Bool(bool),
    Complex,
}

impl Add for RuntimeValue {
    type Output = RuntimeValue;

    fn add(self, rhs: Self) -> Self::Output {
        if let RuntimeValue::Number(l) = self {
            if let RuntimeValue::Number(r) = rhs {
                RuntimeValue::Number(l + r)
            } else {
                RuntimeValue::Null
            }
        } else if let RuntimeValue::String(l) = self {
            if let RuntimeValue::String(r) = rhs {
                RuntimeValue::String(format!("{}{}", l, r))
            } else {
                RuntimeValue::Null
            }
        } else {
            RuntimeValue::Null
        }
    }
}

impl Sub for RuntimeValue {
    type Output = RuntimeValue;

    fn sub(self, rhs: Self) -> Self::Output {
        if let RuntimeValue::Number(l) = self {
            if let RuntimeValue::Number(r) = rhs {
                RuntimeValue::Number(l - r)
            } else {
                RuntimeValue::Null
            }
        } else {
            RuntimeValue::Null
        }
    }
}

impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        if let RuntimeValue::Number(l) = self.clone() {
            if let RuntimeValue::Number(r) = other.clone() {
                l == r
            } else {
                false
            }
        } else if let RuntimeValue::String(l) = self.clone() {
            if let RuntimeValue::String(r) = other.clone() {
                l == r
            } else {
                false
            }
        } else if let RuntimeValue::Bool(l) = self.clone() {
            if let RuntimeValue::Bool(r) = other.clone() {
                l == r
            } else {
                false
            }
        } else {
            false
        }
    }
}