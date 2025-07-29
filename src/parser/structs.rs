use crate::global::DataType;
use crate::interpreter::scope::{FnArgs, FunctionData, RuntimeScopeW};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

#[derive(Clone, PartialEq, Debug)]
pub enum ASTNode {
    Program(Vec<ASTNode>),
    Expression(ExpressionType),
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    VariableDeclaration(bool, String, DataType, Box<ASTNode>),
    Assignment(AssignmentProperty, Box<ASTNode>),
    RepeatOperation(Box<ASTNode>, Box<ASTNode>),
    FunctionDeclaration(String, IndexMap<String, DataType>, Box<ASTNode>, DataType),
    BindingAccess(String),
    CodeBlock(Vec<ASTNode>),
    FunctionCall(Box<ASTNode>, Vec<ASTNode>),
    IfStatement(IfStatement),
    OnceStatement(OnceStatement),
    UseNative(UseNative),
    Misc(MiscNodeType),
    ForStatement(ForStatement),
    ComplexTypeAccessor(String, String),
    EnumDeclaration(String, Vec<String>),
    Typeof(Box<ASTNode>),
    LayoutDeclaration(LayoutDeclaration),
    LayoutCreation(LayoutCreation),
    LayoutFieldAccess(Box<ASTNode>, String),
    MixStatement(String, Vec<ParserFunctionData>),
    InternalMulti(Vec<ASTNode>),
    UseModule(String, String),
    Lambda(IndexMap<String, DataType>, Box<ASTNode>, DataType),
    Indexing(Box<ASTNode>, Box<ASTNode>),
    InternalStop(usize, String),
}

#[derive(Clone, PartialEq, Debug)]
pub enum AssignmentProperty {
    Variable(String),
    LayoutField(Box<ASTNode>, String),
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
    pub else_block: Option<Box<ASTNode>>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct UseNative {
    pub name: String,
    pub from: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LayoutCreation {
    pub name: String,
    pub specified_fields: HashMap<String, Box<ASTNode>>,
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
    pub fields: HashMap<String, FieldParserDescription>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FieldParserDescription {
    pub type_id: String,
    pub default_value: Option<Box<ASTNode>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParserFunctionData {
    pub name: String,
    pub args: FnArgs,
    pub body: Vec<ASTNode>,
    pub return_type: DataType,
    pub tied: bool,
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
    DoubleDot,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ExpressionType {
    Null,
    Binary(Box<BinaryExpression>),
}
