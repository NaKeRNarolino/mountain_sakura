mod structs;

use crate::interpreter::structs::RuntimeValue;
use crate::parser::structs::{ASTNode, BinaryExpression, ExpressionType, Operand};

pub struct Interpreter {
    program: Vec<ASTNode>
}

impl Interpreter {
    pub fn current_node(&mut self) {

    }

    pub fn new(src: ASTNode) -> Self {
        if let ASTNode::Program(program) = src {
            Self {
                program
            }
        } else {
            panic!("Unable to parse program, as it's not an ASTNode::Program");
        }
    }

    pub fn eval_program(&self) -> RuntimeValue {
        let mut last_evaluated = RuntimeValue::Null;

        for node in &self.program {
            last_evaluated = self.eval(node);
        }

        last_evaluated
    }

    fn eval(&self, src: &ASTNode) -> RuntimeValue {
        match src {
            ASTNode::Program(_) => RuntimeValue::Null,
            ASTNode::Expression(expression_type) => self.eval_expression(expression_type),
            ASTNode::Number(v) => RuntimeValue::Number(v.clone()),
            ASTNode::String(_) => RuntimeValue::Null,
            ASTNode::Identifier(_) => RuntimeValue::Null,
        }
    }

    fn eval_expression(&self, expression_type: &ExpressionType) -> RuntimeValue {
        match expression_type {
            ExpressionType::Primary => RuntimeValue::Null,
            ExpressionType::Binary(expression) => self.eval_binary_expression(*expression.clone())
        }
    }

    fn eval_binary_expression(&self, binary_expression: BinaryExpression) -> RuntimeValue {
        match binary_expression.operand {
            Operand::Equality => RuntimeValue::Null,
            Operand::EqArrow => RuntimeValue::Null,
            Operand::DoubleArrow => RuntimeValue::Null,
            Operand::Arrow => RuntimeValue::Null,
            Operand::BackwardArrow => RuntimeValue::Null,
            Operand::ExclamationMk => RuntimeValue::Null,
            Operand::QuestionMk => RuntimeValue::Null,
            Operand::Plus => self.eval_add_expression(binary_expression, false),
            Operand::Minus => self.eval_add_expression(binary_expression, true),
            Operand::Multiply => self.eval_multiply_expression(binary_expression, false),
            Operand::Divide => self.eval_multiply_expression(binary_expression, true),
            Operand::Modulo => RuntimeValue::Null,
            Operand::Increment => RuntimeValue::Null,
            Operand::Decrement => RuntimeValue::Null,
            Operand::Bigger => RuntimeValue::Null,
            Operand::Smaller => RuntimeValue::Null,
            Operand::BiggerEqual => RuntimeValue::Null,
            Operand::SmallerEqual => RuntimeValue::Null,
            Operand::Equal => RuntimeValue::Null
        }
    }

    fn eval_add_expression(&self, binary_expression: BinaryExpression, minus_mode: bool) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = match left {
            ASTNode::Program(_) => RuntimeValue::Null,
            ASTNode::Expression(v) => self.eval_expression(&v),
            ASTNode::Number(v) => RuntimeValue::Number(v),
            ASTNode::String(_) => RuntimeValue::Null,
            ASTNode::Identifier(_) => RuntimeValue::Null
        };

        let right_value = match right {
            ASTNode::Program(_) => RuntimeValue::Null,
            ASTNode::Expression(v) => self.eval_expression(&v),
            ASTNode::Number(v) => RuntimeValue::Number(v),
            ASTNode::String(_) => RuntimeValue::Null,
            ASTNode::Identifier(_) => RuntimeValue::Null
        };

        if let RuntimeValue::Number(l) = left_value {
            if let RuntimeValue::Number(r) = right_value {
                RuntimeValue::Number(l + if minus_mode { -1.0 * r } else { r })
            } else {
                RuntimeValue::Null
            }
        } else {
            RuntimeValue::Null
        }
    }
    
    fn eval_multiply_expression(&self, binary_expression: BinaryExpression, division_mode: bool) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = match left {
            ASTNode::Program(_) => RuntimeValue::Null,
            ASTNode::Expression(v) => self.eval_expression(&v),
            ASTNode::Number(v) => RuntimeValue::Number(v),
            ASTNode::String(_) => RuntimeValue::Null,
            ASTNode::Identifier(_) => RuntimeValue::Null
        };

        let right_value = match right {
            ASTNode::Program(_) => RuntimeValue::Null,
            ASTNode::Expression(v) => self.eval_expression(&v),
            ASTNode::Number(v) => RuntimeValue::Number(v),
            ASTNode::String(_) => RuntimeValue::Null,
            ASTNode::Identifier(_) => RuntimeValue::Null
        };

        if let RuntimeValue::Number(l) = left_value {
            if let RuntimeValue::Number(r) = right_value {
                RuntimeValue::Number(l * if division_mode { 1.0 / r } else { r })
            } else {
                RuntimeValue::Null
            }
        } else {
            RuntimeValue::Null
        }
    }
}