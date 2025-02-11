pub mod environment;
pub mod structs;

use crate::interpreter::environment::{FnArgs, RuntimeScope};
use crate::interpreter::structs::RuntimeValue;
use crate::parser::structs::{ASTNode, BinaryExpression, ExpressionType, IfStatement, OnceStatement, Operand};
use std::cell::RefCell;
use std::ptr::eq;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub struct Interpreter {
    program: Vec<ASTNode>,
}

type RuntimeScopeW = Arc<RwLock<RuntimeScope>>;

impl<'a> Interpreter {
    pub fn new(src: ASTNode) -> Self {
        if let ASTNode::Program(program) = src {
            Self { program }
        } else {
            panic!("Unable to parse program, as it's not an ASTNode::Program");
        }
    }

    pub fn eval_program(&self, scope: RuntimeScope) -> RuntimeValue {
        let mut last_evaluated = RuntimeValue::Null;

        let scope = Arc::new(RwLock::new(scope));

        for node in &self.program {
            last_evaluated = self.eval(node, scope.clone());
        }

        last_evaluated
    }

    fn eval(&self, src: &ASTNode, scope: RuntimeScopeW) -> RuntimeValue {
        match src {
            ASTNode::Program(body) => {
                let mut last_evaluated = RuntimeValue::Null;

                for node in body {
                    last_evaluated = self.eval(node, scope.clone());
                }

                last_evaluated
            }
            ASTNode::Expression(expression_type) => self.eval_expression(expression_type, scope),
            ASTNode::Number(v) => RuntimeValue::Number(v.clone()),
            ASTNode::String(v) => RuntimeValue::String(v.clone()),
            ASTNode::Boolean(v) => RuntimeValue::Bool(v.clone()),
            ASTNode::Identifier(identifier) => self.get_variable(identifier.clone(), scope),
            ASTNode::VariableDeclaration(is_let, identifier, value) => {
                self.eval_variable_declaration(
                    is_let.clone(),
                    identifier.clone(),
                    *value.clone(),
                    scope,
                );
                RuntimeValue::Null
            }
            ASTNode::VariableAssignment(identifier, value) => {
                self.eval_variable_assignment(identifier.clone(), *value.clone(), scope);
                RuntimeValue::Null
            }
            ASTNode::RepeatOperation(count, operation) => {
                self.eval_repeat_operation(*count.clone(), *operation.clone(), scope);
                RuntimeValue::Null
            }
            ASTNode::FunctionDeclaration(identifier, args, body) => {
                if let ASTNode::CodeBlock(body_code) = *body.clone() {
                    self.eval_fn_declaration(identifier.clone(), args.clone(), body_code, scope);
                    RuntimeValue::Null
                } else {
                    unreachable!()
                }
            }
            ASTNode::CodeBlock(code) => {
                self.eval(
                    &ASTNode::Program(code.clone()),
                    scope
                )
            }
            ASTNode::FunctionCall(identifier, args) => {
                self.eval_fn_call(identifier.clone(), args.clone(), scope)
            }
            ASTNode::IfStatement(stmt) => {
                self.eval_if_statement(stmt.clone(), scope)
            },
            ASTNode::Misc(_) => unreachable!(),
            ASTNode::OnceStatement(stmt) => {
                self.eval_once_statement(stmt.clone(), scope)
            },
        }
    }

    fn eval_expression(
        &self,
        expression_type: &ExpressionType,
        scope: RuntimeScopeW,
    ) -> RuntimeValue {
        match expression_type {
            ExpressionType::Null => RuntimeValue::Null,
            ExpressionType::Binary(expression) => {
                self.eval_binary_expression(*expression.clone(), scope)
            }
        }
    }

    fn eval_binary_expression(
        &self,
        binary_expression: BinaryExpression,
        scope: RuntimeScopeW,
    ) -> RuntimeValue {
        match binary_expression.operand {
            Operand::Equality => self.eval_equality_expression(binary_expression, false, scope),
            Operand::Inequality => self.eval_equality_expression(binary_expression, true, scope),
            Operand::EqArrow => RuntimeValue::Null,
            Operand::DoubleArrow => RuntimeValue::Null,
            Operand::Arrow => RuntimeValue::Null,
            Operand::BackwardArrow => RuntimeValue::Null,
            Operand::ExclamationMk => RuntimeValue::Null,
            Operand::QuestionMk => RuntimeValue::Null,
            Operand::Plus => self.eval_add_expression(binary_expression, false, scope),
            Operand::Minus => self.eval_add_expression(binary_expression, true, scope),
            Operand::Multiply => self.eval_multiply_expression(binary_expression, false, scope),
            Operand::Divide => self.eval_multiply_expression(binary_expression, true, scope),
            Operand::Modulo => RuntimeValue::Null,
            Operand::Increment => RuntimeValue::Null,
            Operand::Decrement => RuntimeValue::Null,
            Operand::Bigger => {
                self.eval_comparison_expression(binary_expression, true, false, scope)
            }
            Operand::Smaller => {
                self.eval_comparison_expression(binary_expression, false, false, scope)
            }
            Operand::BiggerEqual => {
                self.eval_comparison_expression(binary_expression, true, true, scope)
            }
            Operand::SmallerEqual => {
                self.eval_comparison_expression(binary_expression, false, false, scope)
            }
            Operand::Equal => RuntimeValue::Null,
        }
    }

    fn eval_add_expression(
        &self,
        binary_expression: BinaryExpression,
        minus_mode: bool,
        scope: RuntimeScopeW,
    ) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = self.eval(&left, scope.clone());

        let right_value = self.eval(&right, scope.clone());

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
        scope: RuntimeScopeW,
    ) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = self.eval(&left, scope.clone());

        let right_value = self.eval(&right, scope.clone());

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
        scope: RuntimeScopeW,
    ) -> RuntimeValue {
        let left = *binary_expression.left;
        let right = *binary_expression.right;

        let left_value = self.eval(&left, scope.clone());

        let right_value = self.eval(&right, scope.clone());

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

    fn get_variable(&self, identifier: String, scope: RuntimeScopeW) -> RuntimeValue {
        scope.read().unwrap().read_variable(identifier).unwrap()
    }

    fn eval_variable_declaration(
        &self,
        is_immut: bool,
        identifier: String,
        value: ASTNode,
        scope: RuntimeScopeW,
    ) {
        let eval = self.eval(&value, scope.clone());
        scope
            .write()
            .unwrap()
            .declare_variable(identifier, eval, is_immut)
    }
    //
    fn eval_variable_assignment(
        &self,
        identifier: String,
        value: ASTNode,
        scope: RuntimeScopeW,
    ) {
        let v = self.eval(&value, scope.clone());
        scope.write().unwrap().assign_variable(identifier, v);
    }
    //
    fn eval_repeat_operation(
        &self,
        count: ASTNode,
        operation: ASTNode,
        scope: RuntimeScopeW,
    ) {
        let count_rv = self.eval(&count, scope.clone());
        if let RuntimeValue::Number(count) = count_rv {
            for _ in 0..count.floor().abs() as u32 {
                self.eval(&operation, scope.clone());
            }
        } else {
            panic!("The value on the right of the repeat operator (?:) cannot be evaluated into a number.");
        }
    }

    fn eval_fn_declaration(
        &self,
        identifier: String,
        args: FnArgs,
        body: Vec<ASTNode>,
        scope: RuntimeScopeW,
    ) {
        scope
            .write()
            .unwrap()
            .declare_function(identifier, args, body);
    }
    //
    fn eval_fn_call(
        &self,
        identifier: String,
        args: Vec<ASTNode>,
        scope: RuntimeScopeW,
    ) -> RuntimeValue {
        let mut new_scope = RuntimeScope::new(Some(scope.clone()));

        let fn_data = new_scope.get_function(identifier).unwrap();

        let mut argu: RuntimeValue = RuntimeValue::Null;

        for (i, (arg, data_type)) in fn_data.args.iter().enumerate() {
            dbg!(arg, &args[i]);
            let ev = self.eval(&args[i], scope.clone());
            argu = ev.clone();
            new_scope.declare_variable(arg.clone(), ev, true);
        }

        let r = self.eval(
            &ASTNode::Program(fn_data.body),
            Arc::new(RwLock::new(new_scope)),
        );
        dbg!(&r, argu);
        r
    }

    fn eval_comparison_expression(
        &self,
        expr: BinaryExpression,
        bigger: bool,
        equal: bool,
        scope: RuntimeScopeW,
    ) -> RuntimeValue {
        let lv = self.eval(&expr.left, scope.clone());
        let rv = self.eval(&expr.right, scope.clone());

        if bigger {
            lv.bigger(&rv, equal)
        } else {
            lv.smaller(&rv, equal)
        }
    }
    
    fn eval_if_statement(
        &self,
        statement: IfStatement,
        scope: RuntimeScopeW,
    ) -> RuntimeValue {
        let eval_stmt = self.eval(&statement.condition, scope.clone());

        if let RuntimeValue::Bool(stmt_value) = eval_stmt {
            if stmt_value {
                self.eval(&statement.if_block, scope)
            } else {
                if statement.else_block.is_some() {
                    self.eval(&statement.else_block.unwrap(), scope)
                } else {
                    RuntimeValue::Null
                }
            }
        } else {
            panic!("Expected a Boolean value as a result")
        }
    }

    fn eval_once_statement(
        &self,
        statement: OnceStatement,
        scope: RuntimeScopeW,
    ) -> RuntimeValue {
        let mut res = RuntimeValue::Null;
        let mut set = false;

        for if_st in statement.if_statements {
            let eval_stmt = self.eval(&if_st.condition, scope.clone());

            if let RuntimeValue::Bool(stmt_value) = eval_stmt {
                if stmt_value {
                    res = self.eval(&if_st.if_block.clone(), scope.clone());
                    set = true;
                    break;
                }
            }
        }

        if !set {
            if statement.else_block.is_some() {
                res = self.eval(&statement.else_block.unwrap(), scope);
            }
        }

        res
    }
}
