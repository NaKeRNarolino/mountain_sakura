use std::sync::Arc;
use crate::interpreter::structs::{MoSaNativeFunction, RuntimeValue};

#[derive(Clone)]
pub struct MoSaBinding {
    pub path: String,
    pub binding: MoSaNativeFunction
}

impl MoSaBinding {
    pub fn new(name: impl Into<String>, binding: impl Fn(Vec<RuntimeValue>) -> RuntimeValue + 'static) -> Self {
        Self {
            path: name.into(), binding: Arc::new(binding)
        }
    }
}

pub trait MoSaNativeGen {
    fn binding(self, name: impl Into<String>) -> MoSaBinding;
}

impl<T> MoSaNativeGen for T
where
    T: Fn(Vec<RuntimeValue>) -> RuntimeValue + 'static {
    fn binding(self, name: impl Into<String>) -> MoSaBinding {
        MoSaBinding::new(name.into(), self)
    }
}