pub mod scope;
pub mod structs;

use crate::interpreter::scope::{FnArgs, RuntimeScope};
use crate::interpreter::structs::{EnumData, IterablePair, LayoutData, RuntimeValue};
use crate::parser::structs::{ASTNode, AssignmentProperty, BinaryExpression, ExpressionType, ForStatement, IfStatement, LayoutCreation, LayoutDeclaration, OnceStatement, Operand, UseNative};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ptr::eq;
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLock};
use uuid::Uuid;
use crate::global::DataType;
use crate::interpreter::structs::ComplexRuntimeValue;

pub struct Interpreter {
    program: Vec<ASTNode>,
}

type RuntimeScopeW = Arc<RwLock<RuntimeScope>>;

impl Interpreter {
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
            ASTNode::VariableDeclaration(is_let, identifier, type_id, value) => {
                self.eval_variable_declaration(
                    is_let.clone(),
                    identifier.clone(),
                    type_id.clone(),
                    *value.clone(),
                    scope,
                );
                RuntimeValue::Null
            }
            ASTNode::Assignment(identifier, value) => {
                self.eval_assignment(identifier.clone(), *value.clone(), scope);
                RuntimeValue::Null
            }
            ASTNode::RepeatOperation(count, operation) => {
                self.eval_repeat_operation(*count.clone(), *operation.clone(), scope);
                RuntimeValue::Null
            }
            ASTNode::FunctionDeclaration(identifier, args, body, data_type) => {
                if let ASTNode::CodeBlock(body_code) = *body.clone() {
                    self.eval_fn_declaration(identifier.clone(), args.clone(), body_code, scope);
                    RuntimeValue::Null
                } else {
                    unreachable!()
                }
            }
            ASTNode::CodeBlock(code) => {
                self.eval_code_block(
                    code.clone(),
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
            ASTNode::UseNative(use_native) => {
                self.eval_define_native_fn(use_native, scope);
                RuntimeValue::Null
            },
            ASTNode::BindingAccess(name) => {
                self.eval_binding_access(name, scope)
            },
            ASTNode::ForStatement(stmt) => {
                self.eval_for_statement(stmt, scope);
                RuntimeValue::Null
            },
            ASTNode::EnumAccessor(enum_id, entry) => self.eval_enum_access(enum_id, entry, scope),
            ASTNode::EnumDeclaration(name, entries) => {
                self.eval_enum_declaration(name, entries, scope);
                RuntimeValue::Null
            },
            ASTNode::Typeof(v) => self.eval_typeof(v, scope),
            ASTNode::LayoutDeclaration(v) => {
                self.eval_layout_declaration(v.clone(), scope);
                RuntimeValue::Null
            },
            ASTNode::LayoutCreation(v) => self.eval_layout_creation(v.clone(), scope),
            ASTNode::LayoutFieldAccess(name, field) => self.eval_layout_field_access(name.clone(), field.clone(), scope)
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
            Operand::DoubleDot => self.eval_double_dot_expressions(binary_expression, scope),
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
        type_id: DataType,
        value: ASTNode,
        scope: RuntimeScopeW,
    ) {
        let eval = self.eval(&value, scope.clone());
        scope
            .write()
            .unwrap()
            .declare_variable(identifier, type_id, eval, is_immut)
    }
    //
    fn eval_assignment(
        &self,
        identifier: AssignmentProperty,
        value: ASTNode,
        scope: RuntimeScopeW,
    ) {
        let v = self.eval(&value, scope.clone());
        if let AssignmentProperty::Variable(id) = identifier {
            scope.write().unwrap().assign_variable(id, v);
        } else if let AssignmentProperty::LayoutField(name, field) = identifier {
            let variable = scope.read().unwrap().read_variable(name.clone()).expect(
                &format!("Cannot find layout variable `{}` in this scope.", &name)
            );

            let data = self.cast_to_layout_data(variable.clone(), &name);

            if !data.entries.read().unwrap().contains_key(&field) {
                panic!("Field `{}` does not exist on type `{}`.", &field, &data.layout_id)
            }

            self.cast_to_layout_data(variable, &name).entries.write().unwrap().insert(field, v);
        }
    }
    //
    fn eval_repeat_operation(
        &self,
        count: ASTNode,
        operation: ASTNode,
        scope: RuntimeScopeW,
    ) {
        let count_rv = self.eval(&count, scope.clone());
        let scope_bound = Arc::new(RwLock::new(RuntimeScope::new(Some(scope))));
        if let RuntimeValue::Number(count) = count_rv {
            for idx in 0..count.floor().abs() as u32 {
                scope_bound.write().unwrap().assign_binding(String::from("index"), RuntimeValue::Number(idx as f64));
                self.eval(&operation, scope_bound.clone());
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
        // dbg!(&&identifier);
        let native = scope.read().unwrap().get_native_function_from_ident(identifier.clone()).is_some();


        if !native {
            let mut new_scope = RuntimeScope::new(Some(scope.clone()));

            let fn_data = new_scope.get_function(identifier).unwrap();

            for (i, (arg, data_type)) in fn_data.args.iter().enumerate() {
                dbg!(arg, &args[i]);
                let ev = self.eval(&args[i], scope.clone());
                let r#type = scope.read().unwrap().get_value_type(&ev);
                if r#type != data_type.clone() {
                    panic!("Cannot pass value of type `{}` to function argument `{}` of type `{}`", r#type, arg, data_type)
                }
                new_scope.declare_variable(arg.clone(), data_type.clone(), ev, true);
            }

            let r = self.eval(
                &ASTNode::Program(fn_data.body),
                Arc::new(RwLock::new(new_scope)),
            );
            r
        } else {
            let args_ev = args.iter().map(|x| self.eval(x, scope.clone())).collect();
            (scope.read().unwrap().get_native_function_from_ident(identifier).expect("Cannot find native fn"))(args_ev)
        }
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

    fn eval_define_native_fn(&self, native_fn: &UseNative, scope: RuntimeScopeW) {
        scope.write().unwrap().define_native_function(
            native_fn.name.clone(),
            native_fn.from.clone()
        )
    }
    
    fn eval_code_block(&self, code: Vec<ASTNode>, scope: RuntimeScopeW) -> RuntimeValue {
        let new_scope = RuntimeScope::new(Some(scope.clone()));
        
        let res = self.eval(
            &ASTNode::Program(code.clone()),
            Arc::new(RwLock::new(new_scope)),
        );
        
        res
    }

    fn eval_binding_access(&self, name: &String, scope: RuntimeScopeW) -> RuntimeValue {
        scope.read().unwrap().get_binding(name).unwrap()
    }

    fn eval_double_dot_expressions(
        &self,
        expr: BinaryExpression,
        scope: RuntimeScopeW,
    ) -> RuntimeValue {
        let lv = self.eval(&expr.left, scope.clone());
        let rv = self.eval(&expr.right, scope.clone());

        let r = rv.cast_number().expect("Cannot get number from the expression.");
        let l = lv.cast_number().expect("Cannot get number from the expression.");

        let mut vec = vec![
            IterablePair {
                index: 0,
                value: RuntimeValue::Null
            }; (r - l).floor() as usize
        ];

        for (idx, val) in vec.iter_mut().enumerate() {
            *val = IterablePair {
                index: idx,
                value: RuntimeValue::Number(l.floor() + idx as f64)
            }
        };

        RuntimeValue::Iterable(vec)
    }

    fn eval_for_statement(&self, stmt: &ForStatement, scope: RuntimeScopeW) {
        let scope_bound = RuntimeScope::arc_rwlock_new(Some(scope.clone()));

        let ev_iterable = self.eval(&stmt.iterable, scope.clone());
        
        let iterable = ev_iterable.cast_iterable().expect("Cannot get iterable from the expression!");

        for val in iterable.iter() {
            scope_bound.write().unwrap().assign_binding(
                String::from("index"), RuntimeValue::Number(val.index as f64)
            );
            scope_bound.write().unwrap().assign_binding(
                String::from("value"), val.value.clone()
            );
            
            self.eval(&stmt.block, scope_bound.clone());
        }
    }

    fn eval_enum_access(&self, enum_id: &String, entry: &String, scope: RuntimeScopeW) -> RuntimeValue {
        if let Some(val) = scope.read().unwrap().get_enum_data(enum_id) {
            if val.entries.contains(entry) {
                RuntimeValue::Complex(
                    ComplexRuntimeValue::Enum(EnumData {
                        enum_id: enum_id.clone(),
                        entry: entry.clone(),
                    })
                )
            } else {
                panic!("Enum `{}` does not have entry named `{}`", enum_id, entry)
            }
        } else {
            panic!("Enum `{}` does not exist in this scope.", enum_id)
        }
    }

    fn eval_enum_declaration(&self, name: &String, entries: &Vec<String>, scope: RuntimeScopeW) {
        scope.write().unwrap().declare_enum(
            name.clone(), entries.clone()
        )
    }

    fn eval_typeof(&self, v: &Box<ASTNode>, scope: RuntimeScopeW) -> RuntimeValue {
        let ev = self.eval(v, scope.clone());

        RuntimeValue::String(scope.read().unwrap().get_value_type(&ev).to_string())
    }

    fn eval_layout_declaration(&self, layout_declaration: LayoutDeclaration, scope: RuntimeScopeW) {
        scope.write().unwrap().declare_layout(layout_declaration)
    }

    fn eval_layout_creation(&self, layout_creation: LayoutCreation, scope: RuntimeScopeW) -> RuntimeValue {
        if scope.read().unwrap().get_layout_declaration(&layout_creation.name).is_none() {
            panic!("Cannot find layout `{}` in current scope.", &layout_creation.name)
        }

        let decl = scope.read().unwrap().get_layout_declaration(&layout_creation.name).unwrap();
        
        let mut fields: HashMap<String, RuntimeValue> = HashMap::new();
        
        for (name, data) in decl.fields.clone() {
            if data.default_value.is_some() {
                let ev = self.eval(&data.default_value.unwrap(), scope.clone());
                fields.insert(name, ev);
            }
        }

        for (name, data) in layout_creation.specified_fields {
            let ev = self.eval(&data, scope.clone());
            
            if !&decl.fields.contains_key(&name) {
                panic!("Field `{}` in layout `{}` does not exist.", name, &layout_creation.name)
            }
            
            fields.insert(name, ev);
        }
        
        for (name, _) in decl.fields.clone() {
            if !fields.contains_key(&name) {
                panic!("Field `{}` in layout `{}` is not defined when creating and does not have a default value.", name, &layout_creation.name)
            }
        }
        
        RuntimeValue::Complex(
            ComplexRuntimeValue::Layout(
                Arc::new(LayoutData {
                    layout_id: layout_creation.name,
                    entries: Arc::new(RwLock::new(fields)),
                })
            )
        )
    }

    fn eval_layout_field_access(&self, name: String, field: String, scope: RuntimeScopeW) -> RuntimeValue {
        let variable = scope.read().unwrap().read_variable(name.clone()).expect(
            &format!("Cannot find layout variable `{}` in this scope.", &name)
        );

        let data = self.cast_to_layout_data(variable, &name);

        data.entries.clone().read().unwrap().get(&field).expect(
            &format!("Field `{}` does not exist on type `{}`.", &field, &data.layout_id)
        ).clone()
    }

    fn cast_to_layout_data(&self, variable: RuntimeValue, name: &String) -> Arc<LayoutData> {
        if let RuntimeValue::Complex(complex) = variable {
            if let ComplexRuntimeValue::Layout(data) = complex {
                data
            } else {
                panic!("Variable `{}` is not a layout.", &name)
            }
        } else {
            panic!("Variable `{}` is not of a complex type.", &name)
        }
    }
}
