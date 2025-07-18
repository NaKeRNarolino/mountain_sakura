use std::collections::{HashMap, HashSet};
use crate::interpreter::scope::FunctionData;

#[derive(Clone, Debug)]
pub struct Module {
    exports: HashSet<ModuleExport>
}

#[derive(Clone, Debug)]
pub enum ModuleExport {
    Function(FunctionData)
}