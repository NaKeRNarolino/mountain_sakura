use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use crate::global::{DataType, PrimitiveDataType};
use crate::interpreter::Interpreter;
use crate::interpreter::scope::{RuntimeScope, RuntimeScopeW};
use crate::interpreter::structs::{MoSaNativeFunction, RuntimeValue};
use crate::modules::{Module, ModuleStorage};
use crate::parser::Parser;
use crate::wrapper::MoSaBinding;

#[derive(Clone)]
pub struct MoSaRunner {
    entry: PathBuf,
    bindings: Vec<MoSaBinding>,
    scope_operations: Vec<Arc<dyn Fn(RuntimeScopeW)>>
}

impl MoSaRunner {
    pub fn new(entry: PathBuf) -> Self {
        Self {
            entry,
            bindings: vec![],
            scope_operations: vec![]
        }
    }

    pub fn add_bindings(&self, bindings: Vec<MoSaBinding>) -> Self {
        let mut b = self.bindings.clone();
        b.append(&mut bindings.clone());

        Self {
            bindings: b,
            ..self.clone()
        }
    }

    pub fn run(&self) -> anyhow::Result<RuntimeValue> {
        let mut rs = RuntimeScope::new(None);

        rs.declare_variable("null".to_string(), DataType::Primitive(PrimitiveDataType::Null), RuntimeValue::Null, true);

        for binding in self.bindings.clone() {
            rs.add_native_function(binding.path, binding.binding)
        }
        
        let read = fs::read_to_string(&self.entry)?;

        let mut root = self.entry.clone();

        root.pop();

        let mut r = root.into_os_string().to_str().unwrap().to_string();

        if !r.ends_with('/') {
            r.push('/');
        }

        let fname = self.entry.as_path().file_name().unwrap().to_str().unwrap().to_string();

        let module = Module::new(fname.strip_suffix(".mosa").unwrap().to_string());

        let module_storage = Arc::new(ModuleStorage::new());

        let mut parser = Parser::new(read, module, module_storage.clone(), r, "".to_string());

        let interpreter = Interpreter::new(parser.gen_ast(), module_storage.clone());

        let v = interpreter.eval_program(rs);

        Ok(v)
    }
}

