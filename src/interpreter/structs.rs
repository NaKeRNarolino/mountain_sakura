use std::collections::HashMap;
use std::fmt::Display;
use std::ops::{Add, Div, Mul, Sub};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(f64),
    Null,
    String(String),
    Bool(bool),
    Iterable(Vec<IterablePair>),
    Complex(ComplexRuntimeValue),
}

#[derive(Debug, Clone)]
pub enum ComplexRuntimeValue {
    Enum(EnumData),
    Layout(Arc<LayoutData>)
}


#[derive(Debug, Clone)]
pub struct IterablePair {
    pub index: usize,
    pub value: RuntimeValue,
}
#[derive(Debug, Clone)]
pub struct EnumData {
    pub enum_id: String,
    pub entry: String
}

#[derive(Debug, Clone)]
pub struct LayoutData {
    pub layout_id: String,
    pub entries: Arc<RwLock<HashMap<String, RuntimeValue>>>
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

impl Mul for RuntimeValue {
    type Output = RuntimeValue;

    fn mul(self, rhs: Self) -> Self::Output {
        if let RuntimeValue::Number(l) = self {
            if let RuntimeValue::Number(r) = rhs {
                RuntimeValue::Number(l * r)
            } else if let RuntimeValue::String(r) = rhs {
                RuntimeValue::String(r.repeat(l.floor() as usize))
            } else {
                RuntimeValue::Null
            }
        } else if let RuntimeValue::String(l) = self {
            if let RuntimeValue::Number(r) = rhs {
                RuntimeValue::String(l.repeat(r.floor() as usize))
            } else {
                RuntimeValue::Null
            }
        } else {
            RuntimeValue::Null
        }
    }
}

impl Div for RuntimeValue {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if let RuntimeValue::Number(l) = self {
            if let RuntimeValue::Number(r) = rhs {
                RuntimeValue::Number(l / r)
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
        } else if let RuntimeValue::Complex(t) = self.clone() {
            if let RuntimeValue::Complex(k) = other.clone() {
                t.eq(&k)
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl RuntimeValue {
    pub fn bigger(&self, other: &Self, equal: bool) -> RuntimeValue {
        if let RuntimeValue::Number(l) = self.clone() {
            if let RuntimeValue::Number(r) = other.clone() {
                RuntimeValue::Bool(l > r || (equal && l == r))
            } else {
                RuntimeValue::Null
            }
        } else {
            RuntimeValue::Null
        }
    }

    pub fn smaller(&self, other: &Self, equal: bool) -> RuntimeValue {
        if let RuntimeValue::Number(l) = self.clone() {
            if let RuntimeValue::Number(r) = other.clone() {
                RuntimeValue::Bool(l < r || (equal && l == r))
            } else {
                RuntimeValue::Null
            }
        } else {
            RuntimeValue::Null
        }
    }
}

impl Display for RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            RuntimeValue::Number(num) => {
                num.to_string()
            },
            RuntimeValue::Null => {
                String::from("null")
            },
            RuntimeValue::String(str) => {
                str.clone()
            },
            RuntimeValue::Bool(bool) => {
                String::from(if *bool {
                    "true"
                } else {
                    "false"
                })
            },
            RuntimeValue::Complex(_) => {
                String::from("Unable to properly convert the value to a string.")
            },
            RuntimeValue::Iterable(v) => format!("{:?}", v)
        };
        write!(f, "{}", str)
    }
}

impl RuntimeValue {
    pub fn cast_number(&self) -> Option<f64> {
        if let RuntimeValue::Number(l) = self {
            Some(l.clone())
        } else {
            None
        }
    }

    pub fn cast_iterable(&self) -> Option<&Vec<IterablePair>> {
        if let RuntimeValue::Iterable(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn cast_string(&self) -> Option<&String> {
        if let RuntimeValue::String(l) = self {
            Some(l)
        } else {
            None
        }
    }

    pub fn cast_bool(&self) -> Option<&bool> {
        if let RuntimeValue::Bool(l) = self {
            Some(l)
        } else {
            None
        }
    }
}

impl PartialEq for ComplexRuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        if let Self::Enum(l) = self.clone() {
            if let Self::Enum(r) = other.clone() {
                l.enum_id == r.enum_id && l.entry == r.entry
            } else {
                false
            }
        } else {
            false
        }
    }
}