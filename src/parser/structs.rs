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
    CodeBlock(Vec<ASTNode>),
    FunctionCall(String, Vec<ASTNode>),
    IfStatement(IfStatement),
    OnceStatement(OnceStatement),
    Misc(MiscNodeType),
}

#[derive(Clone, PartialEq, Debug)]
pub struct OnceStatement {
    pub if_statements: Vec<IfStatement>,
    pub else_block: Option<Box<ASTNode>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct IfStatement {
    pub condition: Box<ASTNode>,
    pub if_block: Box<ASTNode>,
    pub else_block: Option<Box<ASTNode>>
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
