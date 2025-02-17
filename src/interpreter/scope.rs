use crate::interpreter::structs::RuntimeValue;
use crate::lexer::structs::KeywordType::Has;
use crate::parser::structs::ASTNode;
use std::cell::RefCell;
use std::collections::HashMap;
use std::process::id;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use uuid::Uuid;

pub type FnArgs = HashMap<String, String>;

pub struct VariableData {
    pub value: RwLock<RuntimeValue>,
    pub immut: bool,
}

#[derive(Clone)]
pub struct FunctionData {
    pub args: FnArgs,
    pub body: Vec<ASTNode>,
}

pub struct RuntimeScope {
    parent: Option<Arc<RwLock<RuntimeScope>>>,
    variables: HashMap<String, VariableData>,
    functions: HashMap<String, FunctionData>,
    native_functions: HashMap<String, Arc<dyn Fn(Vec<RuntimeValue>)->RuntimeValue>>,
    defined_native_functions: HashMap<String, String>
}

impl RuntimeScope {
    pub fn new(parent: Option<Arc<RwLock<RuntimeScope>>>) -> Self {
        Self {
            parent,
            variables: HashMap::new(),
            functions: Default::default(),
            native_functions: Default::default(),
            defined_native_functions: Default::default(),
        }
    }

    pub fn declare_variable(&mut self, name: String, value: RuntimeValue, is_immut: bool) {
        self.variables.insert(
            name,
            VariableData {
                immut: is_immut,
                value: RwLock::new(value),
            },
        );
    }

    pub fn read_variable(&self, name: String) -> Option<RuntimeValue> {
        if let Some(variable) = self.variables.get(&name) {
            Some(variable.value.read().unwrap().clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.read().unwrap().read_variable(name)
            } else {
                panic!("Cannot read the variable {}, as it's not declared", name)
            }
        }
    }

    pub fn assign_variable(&mut self, name: String, value: RuntimeValue) {
        if let Some(variable) = self.variables.get(&name) {
            if variable.immut {
                panic!(
                    "Cannot reassign the variable {}, as it's declared as immutable",
                    name
                );
            }
            self.variables.insert(
                name,
                VariableData {
                    value: RwLock::new(value),
                    immut: variable.immut,
                },
            );
        } else {
            if let Some(parent) = &self.parent {
                parent.write().unwrap().assign_variable(name, value)
            } else {
                panic!("Cannot assign the variable {}, as it's not declared", name)
            }
        }
    }

    pub fn declare_function(&mut self, name: String, args: FnArgs, body: Vec<ASTNode>) {
        self.functions.insert(name, FunctionData { args, body });
    }

    pub fn get_function(&self, name: String) -> Option<FunctionData> {
        if let Some(function) = self.functions.get(&name) {
            Some(function.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.read().unwrap().get_function(name)
            } else {
                panic!("Cannot read the function {}, as it's not declared", name)
            }
        }
    }

    pub fn add_native_function(&mut self, path: String, function: Arc<dyn Fn(Vec<RuntimeValue>)->RuntimeValue>) {
        self.native_functions.insert(path, function);
    }

    pub fn define_native_function(&mut self, ident: String, path: String) {
        self.defined_native_functions.insert(ident, path);
    }

    pub fn get_defined_name(&self, ident: String) -> Option<String> {
        if let Some(def) = self.defined_native_functions.get(&ident) {
            Some(def.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.read().unwrap().get_defined_name(ident)
            } else {
                None
            }
        }
    }

    pub fn get_native_function_from_ident(&self, ident: String) -> Option<Arc<dyn Fn(Vec<RuntimeValue>)->RuntimeValue>> {
        let def = self.get_defined_name(ident.clone());

        if def.is_none() {
            return None
        }

        if let Some(native_function) = self.native_functions.get(
            &def.unwrap()
        ) {
            Some(native_function.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.read().unwrap().get_native_function_from_ident(ident)
            } else {
                None
            }
        }
    }
}
