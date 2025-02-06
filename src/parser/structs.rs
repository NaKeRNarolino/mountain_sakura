use crate::global::DataType;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, PartialEq, Debug)]
pub enum ASTNode {
    Program(Vec<ASTNode>),
    Expression(ExpressionType),
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    VariableDeclaration(bool, String, Box<ASTNode>),
    VariableAssignment(String, Box<ASTNode>),
    RepeatOperation(Box<ASTNode>, Box<ASTNode>),
    FunctionDeclaration(String, HashMap<String, String>, Box<ASTNode>),
    FunctionBody(Vec<ASTNode>),
    FunctionCall(String, Vec<ASTNode>),
    IfStatement(Vec<ASTNode>, Vec<ASTNode>),
    Misc(MiscNodeType),
}

#[derive(Clone, PartialEq, Debug)]
pub enum MiscNodeType {
    DoubleArrow,
}

#[derive(Clone, PartialEq, Debug)]
pub struct BinaryExpression {
    pub left: Box<ASTNode>,
    pub right: Box<ASTNode>,
    pub operand: Operand,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Operand {
    Equality,
    Inequality,
    EqArrow,
    DoubleArrow,
    Arrow,
    BackwardArrow,
    ExclamationMk,
    QuestionMk,
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
}

#[derive(Clone, PartialEq, Debug)]
pub enum ExpressionType {
    Null,
    Binary(Box<BinaryExpression>),
}
