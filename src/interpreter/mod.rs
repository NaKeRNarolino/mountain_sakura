pub mod environment;
pub mod structs;

use std::cell::RefCell;
use std::rc::Rc;
use uuid::Uuid;
use crate::interpreter::environment::{Environment, EnvironmentMap, FnArgs};
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

    pub fn eval_program(&self, env_map: &mut EnvironmentMap) -> RuntimeValue {
        let mut last_evaluated = RuntimeValue::Null;

        let env_id = env_map.new_environment(None);

        for node in &self.program {
            last_evaluated = self.eval(node, env_map, env_id);
        }

        last_evaluated
    }

    fn eval(&self, src: &ASTNode, env_map: &mut EnvironmentMap, env_id: Uuid) -> RuntimeValue {
        match src {
            ASTNode::Program(body) => {
                let mut last_evaluated = RuntimeValue::Null;

                for node in body {
                    last_evaluated = self.eval(node, env_map, env_id);
                }

                last_evaluated
            },
            ASTNode::Expression(expression_type) => {
                self.eval_expression(expression_type, env_map, env_id)
            }
            ASTNode::Number(v) => RuntimeValue::Number(v.clone()),
            ASTNode::String(v) => RuntimeValue::String(v.clone()),
            ASTNode::Boolean(v) => RuntimeValue::Bool(v.clone()),
            ASTNode::Identifier(identifier) => {
                self.get_variable(identifier.clone(), env_map, env_id)
            },
            ASTNode::VariableDeclaration(is_let, identifier, value) => {
                self.eval_variable_declaration(is_let.clone(), identifier.clone(), *value.clone(), env_map, env_id);
                RuntimeValue::Null
            },
            ASTNode::VariableAssignment(identifier, value) => {
                self.eval_variable_assignment(identifier.clone(), *value.clone(), env_map, env_id);
                RuntimeValue::Null
            }
            ASTNode::RepeatOperation(count, operation) => {
                self.eval_repeat_operation(*count.clone(), *operation.clone(), env_map, env_id);
                RuntimeValue::Null
            },
            ASTNode::FunctionDeclaration(identifier, args , body) => {
                if let ASTNode::FunctionBody(body_code) = *body.clone() {
                    // self.eval_fn_declaration(identifier.clone(), args.clone(), body_code, env_map, env_id);
                    RuntimeValue::Null
                } else {
                    unreachable!()
                }
            },
            ASTNode::FunctionBody(..) => {
                unreachable!();
            }
            ASTNode::FunctionCall(identifier, args) => {
                // self.eval_fn_call(identifier.clone(), args.clone(), env_map, env_id)
                RuntimeValue::Null
            },
        }
    }

    fn eval_expression(
        &self,
        expression_type: &ExpressionType,
        env_map: &mut EnvironmentMap, env_id: Uuid,
    ) -> RuntimeValue {
        match expression_type {
            ExpressionType::Null => RuntimeValue::Null,
            ExpressionType::Binary(expression) => {
                self.eval_binary_expression(*expression.clone(), env_map, env_id)
            }
        }
    }

    fn eval_binary_expression(
        &self,
        binary_expression: BinaryExpression,
        env_map: &mut EnvironmentMap, env_id: Uuid,
    ) -> RuntimeValue {
        match binary_expression.operand {
            Operand::Equality => {
                self.eval_equality_expression(binary_expression, false, env_map, env_id)
            }
            Operand::Inequality => {
                self.eval_equality_expression(binary_expression, true, env_map, env_id)
            }
            Operand::EqArrow => RuntimeValue::Null,
            Operand::DoubleArrow => RuntimeValue::Null,
            Operand::Arrow => RuntimeValue::Null,
            Operand::BackwardArrow => RuntimeValue::Null,
            Operand::ExclamationMk => RuntimeValue::Null,
            Operand::QuestionMk => RuntimeValue::Null,
            Operand::Plus => self.eval_add_expression(binary_expression, false, env_map, env_id),
            Operand::Minus => self.eval_add_expression(binary_expression, true, env_map, env_id),
            Operand::Multiply => {
                self.eval_multiply_expression(binary_expression, false, env_map, env_id)
            }
            Operand::Divide => self.eval_multiply_expression(binary_expression, true, env_map, env_id),
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
        env_map: &mut EnvironmentMap, env_id: Uuid,
    ) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = self.eval(&left, env_map, env_id);

        let right_value = self.eval(&right, env_map, env_id);

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
        env_map: &mut EnvironmentMap, env_id: Uuid,
    ) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = self.eval(&left, env_map, env_id);

        let right_value = self.eval(&right, env_map, env_id);

        if division_mode {
            left_value / right_value
        } else {
            left_value * right_value
        }
    }

    fn eval_equality_expression(
        &self,
        binary_expression: BinaryExpression,
        inequality_mode: bool,
        env_map: &mut EnvironmentMap, env_id: Uuid,
    ) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = self.eval(&left, env_map, env_id);

        let right_value = self.eval(&right, env_map, env_id);

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

    fn get_variable(&self, identifier: String, env_map: &mut EnvironmentMap, env_id: Uuid) -> RuntimeValue {
        env_map.get_environment_box(env_id).borrow_mut().resolve_variable(identifier, env_map).1
    }

    fn eval_variable_declaration(&self, is_let: bool, identifier: String, value: ASTNode, env_map: &mut EnvironmentMap, env_id: Uuid) {
        env_map.get_environment_box(env_id).borrow_mut().declare_variable(identifier, env_map, self.eval(&value, &mut env_map.clone(), env_id), !is_let) // UNSURE TODO
    }

    fn eval_variable_assignment(&self, identifier: String, value: ASTNode, env_map: &mut EnvironmentMap, env_id: Uuid) {
        env_map.get_environment_box(env_id).borrow_mut().assign_variable(identifier, env_map, self.eval(&value, &mut env_map.clone(), env_id))
    }

    fn eval_repeat_operation(&self, count: ASTNode, operation: ASTNode, env_map: &mut EnvironmentMap, env_id: Uuid) {
        let count_rv = self.eval(&count, env_map, env_id);
        if let RuntimeValue::Number(count) = count_rv {
            for _ in 0..count.floor().abs() as u32 {
                self.eval(&operation, env_map, env_id);
            }
        } else {
            panic!("The value on the right of the repeat operator (?:) cannot be evaluated into a number.");
        }
    }

    // fn eval_fn_declaration(&self, identifier: String, args: FnArgs, body: Vec<ASTNode>, env_map: &mut EnvironmentMap, env_id: Uuid) {
    //     environment.declare_fn(identifier, args, body).unwrap()
    // }
    // 
    // fn eval_fn_call(&self, identifier: String, args: Vec<ASTNode>, env_map: &mut EnvironmentMap, env_id: Uuid) -> RuntimeValue {
    //     let mut temp_env = environment.with_self_parent();
    // 
    //     dbg!(temp_env.clone());
    // 
    //     let (fn_args, fn_body) = temp_env.get_fn(&identifier).unwrap();
    // 
    //     dbg!(fn_args.clone(), fn_body.clone());
    // 
    //     for (i, (arg, data_type)) in fn_args.iter().enumerate() {
    //         dbg!(arg, &args[i]);
    //         temp_env.declare_variable(false, arg.clone(), self.eval(&args[i], environment)).unwrap();
    //     }
    // 
    //     self.eval(&ASTNode::Program(fn_body), &mut temp_env)
    // }
}
