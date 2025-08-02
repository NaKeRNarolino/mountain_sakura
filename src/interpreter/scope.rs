use crate::global::{ComplexDataType, ReferenceType};
use crate::global::{DataType, NumType, PrimitiveDataType};
use crate::interpreter::structs::{ComplexRuntimeValue, MoSaNativeFunction, Reference, RuntimeValue};
use crate::modules::ModuleExport;
use crate::parser::structs::{
    ASTNode, FieldParserDescription, LayoutDeclaration, ParserFunctionData,
};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::process::exit;
use std::sync::{Arc, RwLock};
use crate::err;

pub type FnArgs = IndexMap<String, DataType>;

pub type RuntimeScopeW = Arc<RwLock<RuntimeScope>>;

#[derive(Debug)]
pub struct VariableData {
    pub value: RwLock<RuntimeValue>,
    pub type_id: DataType,
    pub immut: bool,
}

#[derive(Clone, Debug)]
pub struct FunctionData {
    pub name: String,
    pub args: FnArgs,
    pub body: Vec<ASTNode>,
    pub return_type: DataType,
    pub scope: RuntimeScopeW,
    pub accesses: HashSet<String>,
    pub tied: bool,
}

#[derive(Clone, Debug)]
pub struct EnumDefinition {
    pub name: String,
    pub entries: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ScopeLayoutDeclaration {
    pub name: String,
    pub fields: HashMap<String, FieldParserDescription>,
    pub mixed: Arc<RwLock<HashMap<String, FunctionData>>>,
}

// #[derive(Debug)]
pub struct RuntimeScope {
    parent: Option<Arc<RwLock<RuntimeScope>>>,
    variables: HashMap<String, VariableData>,
    functions: HashMap<String, FunctionData>,
    native_functions: HashMap<String, MoSaNativeFunction>,
    defined_native_functions: HashMap<String, String>,
    bindings: HashMap<String, RuntimeValue>,
    enums: HashMap<String, EnumDefinition>,
    layouts: HashMap<String, Arc<ScopeLayoutDeclaration>>,
    imports: RwLock<HashMap<String, ModuleExport>>,
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
            imports: Default::default(),
        }
    }

    pub fn arc_rwlock_new(parent: Option<Arc<RwLock<RuntimeScope>>>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new(parent)))
    }

    pub fn declare_variable(
        &mut self,
        name: String,
        type_id: DataType,
        value: RuntimeValue,
        is_immut: bool,
    ) {
        let value_type = self.get_value_type(&value);
        if type_id != DataType::InternalInfer && !type_id.matches(&value_type) {
            err!(intrp
                "Cannot declare variable `{}` of type `{}` with value of type `{}`",
                &name, &type_id, value_type
            );
            exit(100)
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
                err!(intrp
                    "Cannot reassign the variable {}, as it's declared as immutable",
                    name
                );
                exit(100);
            }
            let value_type = self.get_value_type(&value);
            if !variable.type_id.matches(&value_type) {
                err!(intrp
                    "Cannot assign value of type `{}` to variable `{}` of type `{}`.",
                    value_type, &name, &variable.type_id
                );
                exit(100)
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
                err!(intrp "Cannot assign the variable {}, as it's not declared", name);
                exit(100);
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
                err!(intrp "Cannot find binding `^{}` in this context.", name);
                exit(100);
            }
        }
    }

    pub fn declare_function(
        scope: RuntimeScopeW,
        name: String,
        args: FnArgs,
        body: Vec<ASTNode>,
        return_type: DataType,
    ) {
        let accesses = scope
            .read()
            .unwrap()
            .variables
            .keys()
            .map(|k| k.clone())
            .collect();
        let fd = FunctionData {
            name: name.clone(),
            args,
            body,
            tied: false,
            return_type,
            scope: scope.clone(),
            accesses,
        };
        scope.write().unwrap().functions.insert(name.clone(), fd);
    }

    pub fn get_function(&self, name: String) -> Option<FunctionData> {
        if let Some(function) = self.functions.get(&name) {
            Some(function.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.read().unwrap().get_function(name)
            } else {
                if let Some(x) = &self.get_import(&name) {
                    if let ModuleExport::Function(fd) = x {
                        Some(fd.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn add_native_function(
        &mut self,
        path: String,
        function: Arc<dyn Fn(Vec<RuntimeValue>) -> RuntimeValue>,
    ) {
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

    pub fn get_native_function_from_ident(
        &self,
        ident: String,
    ) -> Option<Arc<dyn Fn(Vec<RuntimeValue>) -> RuntimeValue>> {
        let def = self.get_defined_name(ident.clone());

        if def.is_none() {
            return None;
        }

        if let Some(native_function) = self.native_functions.get(&def.unwrap()) {
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
            RuntimeValue::Number(_) => {
                DataType::Primitive(PrimitiveDataType::Num(NumType::Dynamic))
            }
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
            }
            RuntimeValue::Iterable(_) => {
                DataType::Primitive(PrimitiveDataType::Iterable(Box::new(DataType::Primitive(
                    PrimitiveDataType::Num(NumType::Dynamic),
                ))))
            }
            RuntimeValue::Reference(v) => match v {
                Reference::Function(_) => DataType::Reference(ReferenceType::Function),
                Reference::MethodLikeFunction(_, _, _) => {
                    DataType::Reference(ReferenceType::Function)
                }
            },
        }
    }

    pub fn declare_enum(&mut self, name: String, entries: Vec<String>) {
        self.enums
            .insert(name.clone(), EnumDefinition { name, entries });
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
        self.layouts.insert(
            layout_info.name.clone(),
            Arc::new(ScopeLayoutDeclaration {
                name: layout_info.name,
                fields: layout_info.fields,
                mixed: Arc::new(RwLock::new(HashMap::new())),
            }),
        );
    }

    pub fn get_layout_declaration(&self, name: &String) -> Option<Arc<ScopeLayoutDeclaration>> {
        if let Some(layout) = self.layouts.get(name) {
            Some(layout.clone())
        } else {
            if let Some(parent) = &self.parent {
                parent.read().unwrap().get_layout_declaration(name)
            } else {
                if let Some(x) = &self.get_import(&name) {
                    if let ModuleExport::Layout(ld) = x {
                        Some(ld.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    pub fn mix_into_layout(
        scope: RuntimeScopeW,
        layout_id: String,
        mix_data: Vec<ParserFunctionData>,
    ) {
        match scope.read().unwrap().layouts.get(&layout_id) {
            None => {
                err!(intrp "Cannot mix into non-existent layout `{}`", layout_id);
                exit(100)
            }
            Some(v) => {
                let mut hm: HashMap<String, FunctionData> = HashMap::new();

                for data in mix_data {
                    hm.insert(
                        data.name.clone(),
                        FunctionData {
                            name: data.name.clone(),
                            args: data.args.clone(),
                            body: data.body.clone(),
                            return_type: data.return_type.clone(),
                            tied: data.tied.clone(),
                            scope: scope.clone(),
                            accesses: {
                                scope
                                    .read()
                                    .unwrap()
                                    .variables
                                    .keys()
                                    .map(|x| x.clone())
                                    .collect()
                            },
                        },
                    );
                }

                v.mixed.write().unwrap().extend(hm)
            }
        }
    }

    pub fn get_import(&self, symbol: &String) -> Option<ModuleExport> {
        self.imports.read().unwrap().get(symbol).cloned()
    }

    pub fn import(&self, symbol: String, export: ModuleExport) {
        self.imports.write().unwrap().insert(symbol, export);
    }

    pub fn preplace_native_functions(&mut self, natives: HashMap<String, MoSaNativeFunction>) {
        self.native_functions = natives
    }
    
    pub fn get_native_functions(&self) -> HashMap<String, MoSaNativeFunction> {
        self.native_functions.clone()
    }
}
