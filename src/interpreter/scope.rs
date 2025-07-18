use crate::global::{ComplexDataType, ReferenceType};
use crate::global::{DataType, NumType, PrimitiveDataType};
use crate::interpreter::structs::{ComplexRuntimeValue, Reference, RuntimeValue};
use crate::parser::structs::{ASTNode, FieldParserDescription, LayoutDeclaration};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, RwLock};
use indexmap::IndexMap;

pub type FnArgs = IndexMap<String, DataType>;

pub type RuntimeScopeW = Arc<RwLock<RuntimeScope>>;


#[derive(Debug)]
pub struct VariableData {
    pub value: RwLock<RuntimeValue>,
    pub type_id: DataType,
    pub immut: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FunctionData {
    pub name: String,
    pub args: FnArgs,
    pub body: Vec<ASTNode>,
    pub return_type: DataType,
    pub tied: bool
}

#[derive(Clone, Debug)]
pub struct EnumDefinition {
    pub name: String,
    pub entries: Vec<String>
}

#[derive(Clone, Debug)]
pub struct ScopeLayoutDeclaration {
    pub name: String,
    pub fields: HashMap<String, FieldParserDescription>,
    pub mixed: Arc<RwLock<HashMap<String, FunctionData>>>
}

// #[derive(Debug)]
pub struct RuntimeScope {
    parent: Option<Arc<RwLock<RuntimeScope>>>,
    variables: HashMap<String, VariableData>,
    functions: HashMap<String, FunctionData>,
    native_functions: HashMap<String, Arc<dyn Fn(Vec<RuntimeValue>)->RuntimeValue>>,
    defined_native_functions: HashMap<String, String>,
    bindings: HashMap<String, RuntimeValue>,
    enums: HashMap<String, EnumDefinition>,
    layouts: HashMap<String, ScopeLayoutDeclaration>,
}

impl Debug for RuntimeScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("/* an instance of RuntimeScope */")
    }
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

    pub fn declare_variable(&mut self, name: String, type_id: DataType, value: RuntimeValue, is_immut: bool) {
        let value_type = self.get_value_type(&value);
        if type_id != DataType::InternalInfer && !type_id.matches(&value_type) {
            panic!("Cannot declare variable `{}` of type `{}` with value of type `{}`", &name, &type_id, value_type)
        }
        self.variables.insert(
            name,
            VariableData {
                immut: is_immut,
                type_id: if type_id == DataType::InternalInfer {
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
                None
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

    pub fn declare_function(&mut self, name: String, args: FnArgs, body: Vec<ASTNode>, return_type: DataType) {
        self.functions.insert(name.clone(), FunctionData { name, args, body, tied: false, return_type });
    }

    pub fn get_function(&self, name: String) -> Option<FunctionData> {
        if let Some(function) = self.functions.get(&name) {
            Some(function.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.read().unwrap().get_function(name)
            } else {
                None
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

    pub fn get_value_type(&self, value: &RuntimeValue) -> DataType {
        match value {
            RuntimeValue::Number(_) => DataType::Primitive(PrimitiveDataType::Num(NumType::Dynamic)),
            RuntimeValue::Null => DataType::Primitive(PrimitiveDataType::Null),
            RuntimeValue::String(_) => DataType::Primitive(PrimitiveDataType::Str),
            RuntimeValue::Bool(_) => DataType::Primitive(PrimitiveDataType::Bool),
            RuntimeValue::Complex(x) => {
                if let ComplexRuntimeValue::Enum(ed) = x {
                    DataType::Complex(ComplexDataType::LayoutOrEnum(ed.enum_id.clone()))
                } else if let ComplexRuntimeValue::Layout(ld) = x {
                    DataType::Complex(ComplexDataType::LayoutOrEnum(ld.layout_id.clone()))
                } else {
                    DataType::Complex(ComplexDataType::Indefinite)
                }
            },
            RuntimeValue::Iterable(_) => DataType::Primitive(PrimitiveDataType::Iterable(
                Box::new(DataType::Primitive(PrimitiveDataType::Num(NumType::Dynamic)))
            )),
            RuntimeValue::Reference(v) => match v {
                Reference::Function(_) => {
                    DataType::Reference(ReferenceType::Function)
                },
                Reference::MethodLikeFunction(_, _, _) => DataType::Reference(ReferenceType::Function)
            }
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
        self.layouts.insert(layout_info.name.clone(), ScopeLayoutDeclaration {
            name: layout_info.name,
            fields: layout_info.fields,
            mixed: Arc::new(RwLock::new(HashMap::new()))
        });
    }


    pub fn get_layout_declaration(&self, name: &String) -> Option<ScopeLayoutDeclaration> {
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

    pub fn mix_into_layout(&self, layout_id: String, mix_data: Vec<FunctionData>) {
        match self.layouts.get(&layout_id) {
            None => {
                panic!("Cannot mix into non-existent layout `{}`", layout_id)
            }
            Some(v) => {
                let mut hm: HashMap<String, FunctionData> = HashMap::new();

                for data in mix_data {
                    hm.insert(data.name.clone(), data);
                }

                v.mixed.write().unwrap().extend(hm)
            }
        }
    }
}
