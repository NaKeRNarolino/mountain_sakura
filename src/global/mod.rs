use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Primitive(PrimitiveDataType),
    Complex(ComplexDataType),
    Reference(ReferenceType),
    InternalInfer,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ComplexDataType {
    LayoutOrEnum(String),
    Indefinite,
}

impl DataType {
    pub fn from_str(string: String, generics: Vec<DataType>) -> Self {
        match string.as_str() {
            "num" => DataType::Primitive(PrimitiveDataType::Num(NumType::Dynamic)),
            "str" => DataType::Primitive(PrimitiveDataType::Str),
            "null" => DataType::Primitive(PrimitiveDataType::Null),
            "bool" => DataType::Primitive(PrimitiveDataType::Bool),
            "iterable" => DataType::Primitive(PrimitiveDataType::Iterable(Box::new(
                generics[0].clone(),
            ))),
            v => DataType::Complex(ComplexDataType::LayoutOrEnum(v.to_string())),
        }
    }

    pub fn matches(&self, data_type: &DataType) -> bool {
        if let DataType::Primitive(PrimitiveDataType::Nullable(dt)) = self.clone() {
            data_type.clone() == DataType::Primitive(PrimitiveDataType::Null)
                || (*dt).clone() == data_type.clone()
        } else {
            data_type.clone() == self.clone()
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            DataType::Primitive(primitive) => match primitive {
                PrimitiveDataType::Num(_) => "num",
                PrimitiveDataType::Iterable(g) => &format!("iterable[{}]", g.to_string()),
                PrimitiveDataType::Str => "str",
                PrimitiveDataType::Bool => "bool",
                PrimitiveDataType::Nullable(v) => &format!("nul {}", (&*v).clone()),
                PrimitiveDataType::Null => "null",
            },
            DataType::Complex(c) => match c {
                ComplexDataType::LayoutOrEnum(v) => v,
                ComplexDataType::Indefinite => "indefinite",
            },
            DataType::InternalInfer => unreachable!(),
            DataType::Reference(v) => match v {
                ReferenceType::Function => "ref[function]",
                ReferenceType::Null => "ref[null]",
            },
        };
        write!(f, "{}", str)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveDataType {
    Num(NumType),
    Iterable(Box<DataType>),
    Str,
    Bool,
    Nullable(Box<DataType>),
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReferenceType {
    Function,
    Null,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NumType {
    Dynamic,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    U8,
    U16,
    U32,
    U64,
}
