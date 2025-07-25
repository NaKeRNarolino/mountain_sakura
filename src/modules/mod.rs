use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use crate::interpreter::scope::{FunctionData, RuntimeScope, RuntimeScopeW, ScopeLayoutDeclaration};
use crate::interpreter::structs::RuntimeValue;
use crate::parser::structs::{ASTNode, LayoutDeclaration, ParserFunctionData};

pub struct ModuleStorage {
    storage: Arc<RwLock<HashMap<String, Module>>>
}

impl ModuleStorage {
    pub fn new() -> ModuleStorage {
        ModuleStorage {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn push(&self, module: Module) {
        if self.get(&module.name).is_none() {
            // !("Module `{}` is already defined", &module.name)
            self.storage.write().unwrap().insert(module.name.clone(), module);
        }
    }

    pub fn get(&self, name: &String) -> Option<Module> {
        self.storage.read().unwrap().get(name).cloned()
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    unmodulated_exported_functions: Arc<RwLock<HashMap<String, ParserFunctionData>>>,
    unmodulated_exported_layouts: Arc<RwLock<HashMap<String, LayoutDeclaration>>>,
    exports: Arc<RwLock<HashMap<String, ModuleExport>>>,
    ast: Arc<RwLock<Vec<ASTNode>>>,
    name: String,
    pub scope: RuntimeScopeW,
    cached_result: Arc<RwLock<Option<RuntimeValue>>>
}

#[derive(Clone, Debug)]
pub enum ModuleExport {
    Function(FunctionData),
    Layout(Arc<ScopeLayoutDeclaration>)
}

impl Module {
    pub fn new(name: String) -> Module {
        Module {
            exports: Arc::new(RwLock::new(HashMap::new())),
            ast: Arc::new(RwLock::new(vec![])),
            name,
            scope: RuntimeScope::arc_rwlock_new(None),
            cached_result: Arc::new(RwLock::new(None)),
            unmodulated_exported_functions: Arc::new(RwLock::new(HashMap::new())),
            unmodulated_exported_layouts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn set_ast(&self, ast: Vec<ASTNode>) {
        *self.ast.write().unwrap() = ast;
    }

    pub fn exports(&self) -> HashMap<String, ModuleExport> {
        self.exports.read().unwrap().clone()
    }

    pub fn ast(&self) -> Vec<ASTNode> {
        self.ast.read().unwrap().clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn push(&self, symbol: String, export: ModuleExport) {
        self.exports.write().unwrap().insert(symbol, export);
    }

    pub fn push_unmodulated_fn(&self, symbol: String, fun: ParserFunctionData) {
        self.unmodulated_exported_functions.write().unwrap().insert(symbol, fun);
    }
    
    pub fn push_unmodulated_layout(&self, symbol: String, lay: LayoutDeclaration) {
        self.unmodulated_exported_layouts.write().unwrap().insert(symbol, lay);
    }

    pub fn unmodulated_exported_functions(&self) -> HashMap<String, ParserFunctionData> {
        self.unmodulated_exported_functions.read().unwrap().clone()
    }

    pub fn unmodulated_exported_layouts(&self) -> HashMap<String, LayoutDeclaration> {
        self.unmodulated_exported_layouts.read().unwrap().clone()
    }

    pub fn has_cache(&self) -> bool {
        self.cached_result.read().unwrap().is_some()
    }

    pub fn cached_result(&self) -> Option<RuntimeValue> {
        self.cached_result.read().unwrap().clone()
    }

    pub fn cache(&self, result: RuntimeValue) {
        *self.cached_result.write().unwrap() = Some(result);
    }

    pub fn scope(&self) -> RuntimeScopeW {
        self.scope.clone()
    }
}