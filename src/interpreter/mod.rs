pub mod environment;
pub mod structs;

use crate::interpreter::environment::Environment;
use crate::interpreter::structs::RuntimeValue;
use crate::parser::structs::{ASTNode, BinaryExpression, ExpressionType, Operand};


pub struct Interpreter {
    program: Vec<ASTNode>,
}

impl Interpreter {
    pub fn new(src: ASTNode) -> Self {
        if let ASTNode::Program(program) = src {
            Self { program }
        } else {
            panic!("Unable to parse program, as it's not an ASTNode::Program");
        }
    }

    pub fn eval_program(&self, environment: &mut Environment) -> RuntimeValue {
        let mut last_evaluated = RuntimeValue::Null;

        for node in &self.program {
            last_evaluated = self.eval(node, environment);
        }

        last_evaluated
    }

    fn eval(&self, src: &ASTNode, environment: &mut Environment) -> RuntimeValue {
        match src {
            ASTNode::Program(_) => RuntimeValue::Null,
            ASTNode::Expression(expression_type) => {
                self.eval_expression(expression_type, environment)
            }
            ASTNode::Number(v) => RuntimeValue::Number(v.clone()),
            ASTNode::String(v) => RuntimeValue::String(v.clone()),
            ASTNode::Boolean(v) => RuntimeValue::Bool(v.clone()),
            ASTNode::Identifier(identifier) => {
                self.get_variable(identifier.clone(), environment)
            },
            ASTNode::VariableDeclaration(is_let, identifier, value) => {
                self.eval_variable_declaration(is_let.clone(), identifier.clone(), *value.clone(), environment);
                RuntimeValue::Null
            },
            ASTNode::VariableAssignment(identifier, value) => {
                self.eval_variable_assignment(identifier.clone(), *value.clone(), environment);
                RuntimeValue::Null
            }
        }
    }

    fn eval_expression(
        &self,
        expression_type: &ExpressionType,
        environment: &mut Environment,
    ) -> RuntimeValue {
        match expression_type {
            ExpressionType::Null => RuntimeValue::Null,
            ExpressionType::Binary(expression) => {
                self.eval_binary_expression(*expression.clone(), environment)
            }
        }
    }

    fn eval_binary_expression(
        &self,
        binary_expression: BinaryExpression,
        environment: &mut Environment,
    ) -> RuntimeValue {
        match binary_expression.operand {
            Operand::Equality => {
                self.eval_equality_expression(binary_expression, false, environment)
            }
            Operand::Inequality => {
                self.eval_equality_expression(binary_expression, true, environment)
            }
            Operand::EqArrow => RuntimeValue::Null,
            Operand::DoubleArrow => RuntimeValue::Null,
            Operand::Arrow => RuntimeValue::Null,
            Operand::BackwardArrow => RuntimeValue::Null,
            Operand::ExclamationMk => RuntimeValue::Null,
            Operand::QuestionMk => RuntimeValue::Null,
            Operand::Plus => self.eval_add_expression(binary_expression, false, environment),
            Operand::Minus => self.eval_add_expression(binary_expression, true, environment),
            Operand::Multiply => {
                self.eval_multiply_expression(binary_expression, false, environment)
            }
            Operand::Divide => self.eval_multiply_expression(binary_expression, true, environment),
            Operand::Modulo => RuntimeValue::Null,
            Operand::Increment => RuntimeValue::Null,
            Operand::Decrement => RuntimeValue::Null,
            Operand::Bigger => RuntimeValue::Null,
            Operand::Smaller => RuntimeValue::Null,
            Operand::BiggerEqual => RuntimeValue::Null,
            Operand::SmallerEqual => RuntimeValue::Null,
            Operand::Equal => RuntimeValue::Null,
        }
    }

    fn eval_add_expression(
        &self,
        binary_expression: BinaryExpression,
        minus_mode: bool,
        environment: &mut Environment,
    ) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = self.eval(&left, environment);

        let right_value = self.eval(&right, environment);

        if minus_mode {
            left_value - right_value
        } else {
            left_value + right_value
        }
    }

    fn eval_multiply_expression(
        &self,
        binary_expression: BinaryExpression,
        division_mode: bool,
        environment: &mut Environment,
    ) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = self.eval(&left, environment);

        let right_value = self.eval(&right, environment);

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

    fn eval_equality_expression(
        &self,
        binary_expression: BinaryExpression,
        inequality_mode: bool,
        environment: &mut Environment,
    ) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = self.eval(&left, environment);

        let right_value = self.eval(&right, environment);

        if inequality_mode {
            RuntimeValue::Bool(left_value != right_value)
        } else {
            RuntimeValue::Bool(left_value == right_value)
        }

        // if let RuntimeValue::Number(l) = left_value {
        //     if let RuntimeValue::Number(r) = right_value {
        //         RuntimeValue::Bool(if !inequality_mode { l == r } else { l != r })
        //     } else {
        //         RuntimeValue::Null
        //     }
        // } else {
        //     RuntimeValue::Null
        // }
    }

    fn get_variable(&self, identifier: String, environment: &mut Environment) -> RuntimeValue {
        if let Ok(var) = environment.get_variable(identifier) {
            var
        } else {
            RuntimeValue::Null
        }
    }

    fn eval_variable_declaration(&self, is_let: bool, identifier: String, value: ASTNode, environment: &mut Environment) {
        environment.declare_variable(!is_let, identifier, self.eval(&value, &mut environment.clone())).unwrap()
    }

    fn eval_variable_assignment(&self, identifier: String, value: ASTNode, environment: &mut Environment) {
        environment.set_variable(identifier, self.eval(&value, &mut environment.clone())).unwrap()
    }
}
