use crate::global::DataType;
use std::collections::HashMap;
use std::hash::Hash;
use crate::interpreter::structs::RuntimeValue;

#[derive(Clone, PartialEq, Debug)]
pub enum ASTNode {
    Program(Vec<ASTNode>),
    Expression(ExpressionType),
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    VariableDeclaration(bool, String, String, Box<ASTNode>),
    Assignment(AssignmentProperty, Box<ASTNode>),
    RepeatOperation(Box<ASTNode>, Box<ASTNode>),
    FunctionDeclaration(String, HashMap<String, String>, Box<ASTNode>),
    BindingAccess(String),
    CodeBlock(Vec<ASTNode>),
    FunctionCall(String, Vec<ASTNode>),
    IfStatement(IfStatement),
    OnceStatement(OnceStatement),
    UseNative(UseNative),
    Misc(MiscNodeType),
    ForStatement(ForStatement),
    EnumAccessor(String, String),
    EnumDeclaration(String, Vec<String>),
    Typeof(Box<ASTNode>),
    LayoutDeclaration(LayoutDeclaration),
    LayoutCreation(LayoutCreation),
    LayoutFieldAccess(String, String)
}


#[derive(Clone, PartialEq, Debug)]
pub enum AssignmentProperty {
    Variable(String),
    LayoutField(String, String)
}

#[derive(Clone, PartialEq, Debug)]
pub struct OnceStatement {
    pub if_statements: Vec<IfStatement>,
    pub else_block: Option<Box<ASTNode>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ForStatement {
    pub iterable: Box<ASTNode>,
    pub block: Box<ASTNode>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct IfStatement {
    pub condition: Box<ASTNode>,
    pub if_block: Box<ASTNode>,
    pub else_block: Option<Box<ASTNode>>
}

#[derive(Clone, PartialEq, Debug)]
pub struct UseNative {
    pub name: String,
    pub from: String
}

#[derive(Clone, PartialEq, Debug)]
pub struct LayoutCreation {
    pub name: String,
    pub specified_fields: HashMap<String, Box<ASTNode>>
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

#[derive(Clone, PartialEq, Debug)]
pub struct LayoutDeclaration {
    pub name: String,
    pub fields: HashMap<String, FieldParserDescription>
}

#[derive(Clone, PartialEq, Debug)]
pub struct FieldParserDescription {
    pub type_id: String,
    pub default_value: Option<Box<ASTNode>>
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
    DoubleDot
}

#[derive(Clone, PartialEq, Debug)]
pub enum ExpressionType {
    Null,
    Binary(Box<BinaryExpression>),
}
