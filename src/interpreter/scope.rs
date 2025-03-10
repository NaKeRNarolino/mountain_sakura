use std::alloc::Layout;
use crate::interpreter::structs::{ComplexRuntimeValue, LayoutData, RuntimeValue};
use crate::lexer::structs::KeywordType::Has;
use crate::parser::structs::{ASTNode, LayoutDeclaration};
use std::cell::RefCell;
use std::collections::HashMap;
use std::process::id;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use uuid::Uuid;

pub type FnArgs = HashMap<String, String>;

pub struct VariableData {
    pub value: RwLock<RuntimeValue>,
    pub type_id: String,
    pub immut: bool,
}

#[derive(Clone)]
pub struct FunctionData {
    pub args: FnArgs,
    pub body: Vec<ASTNode>,
}

#[derive(Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub entries: Vec<String>
}

pub struct RuntimeScope {
    parent: Option<Arc<RwLock<RuntimeScope>>>,
    variables: HashMap<String, VariableData>,
    functions: HashMap<String, FunctionData>,
    native_functions: HashMap<String, Arc<dyn Fn(Vec<RuntimeValue>)->RuntimeValue>>,
    defined_native_functions: HashMap<String, String>,
    bindings: HashMap<String, RuntimeValue>,
    enums: HashMap<String, EnumDefinition>,
    layouts: HashMap<String, LayoutDeclaration>,
}

impl RuntimeScope {
    pub fn new(parent: Option<Arc<RwLock<RuntimeScope>>>) -> Self {
        Self {
            parent,
            variables: HashMap::new(),
            functions: Default::default(),
            native_functions: Default::default(),
            defined_native_functions: Default::default(),
            bindings: Default::default(),
            enums: Default::default(),
            layouts: Default::default(),
        }
    }
    
    pub fn arc_rwlock_new(parent: Option<Arc<RwLock<RuntimeScope>>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new(parent)))
    }

    pub fn declare_variable(&mut self, name: String, type_id: String, value: RuntimeValue, is_immut: bool) {
        let value_type = self.get_value_type(&value);
        if type_id.clone() != "1MOSA_UNDEFINED" && type_id.clone() != value_type {
            panic!("Cannot declare variable `{}` of type `{}` with value of type `{}`", &name, &type_id, value_type)
        }
        self.variables.insert(
            name,
            VariableData {
                immut: is_immut,
                type_id: if type_id == "1MOSA_UNDEFINED" {
                    value_type
                } else {
                    type_id.clone()
                },
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
            let value_type = self.get_value_type(&value);
            if variable.type_id.clone() != value_type {
                panic!("Cannot assign value of type `{}` to variable `{}` of type `{}`.", value_type, &name, &variable.type_id)
            }
            self.variables.insert(
                name,
                VariableData {
                    value: RwLock::new(value.clone()),
                    type_id: variable.type_id.clone(),
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

    pub fn assign_binding(&mut self, name: String, value: RuntimeValue) {
        self.bindings.insert(name, value);
    }

    pub fn get_binding(&self, name: &String) -> Option<RuntimeValue> {
        if let Some(binding) = self.bindings.get(name) {
            Some(binding.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.read().unwrap().get_binding(name)
            } else {
                panic!("Cannot find binding ^{} in this context.", name)
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

    pub fn get_value_type(&self, value: &RuntimeValue) -> String {
        match value {
            RuntimeValue::Number(_) => { "num".to_string() }
            RuntimeValue::Null => { "null".to_string() }
            RuntimeValue::String(_) => { "str".to_string() }
            RuntimeValue::Bool(_) => { "bool".to_string() }
            RuntimeValue::Complex(x) => {
                if let ComplexRuntimeValue::Enum(ed) = x {
                    ed.enum_id.clone()
                } else if let ComplexRuntimeValue::Layout(ld) = x {
                    ld.layout_id.clone()
                } else {
                    "complex".to_string()
                }
            },
            RuntimeValue::Iterable(_) => { "iterable".to_string() }
        }
    }

    pub fn declare_enum(&mut self, name: String, entries: Vec<String>) {
        self.enums.insert(name.clone(), EnumDefinition {
            name,
            entries,
        });
    }

    pub fn get_enum_data(&self, name: &String) -> Option<EnumDefinition> {
        if let Some(def) = self.enums.get(name) {
            Some(def.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.read().unwrap().get_enum_data(name)
            } else {
                None
            }
        }
    }

    pub fn declare_layout(&mut self, layout_info: LayoutDeclaration) {
        self.layouts.insert(layout_info.name.clone(), layout_info);
    }

    pub fn get_layout_declaration(&self, name: &String) -> Option<LayoutDeclaration> {
        if let Some(layout) = self.layouts.get(name) {
            Some(layout.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.read().unwrap().get_layout_declaration(name)
            } else {
                None
            }
        }
    }
}
