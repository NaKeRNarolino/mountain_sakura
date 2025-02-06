#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Num(NumType),
    String,
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
