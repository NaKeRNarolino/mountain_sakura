use crate::interpreter::structs::RuntimeValue;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    pub parent: Option<Box<Environment>>,
    pub variables: HashMap<String, (bool, RuntimeValue)>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
        }
    }

    pub fn declare_variable(
        &mut self,
        is_const: bool,
        identifier: String,
        value: RuntimeValue,
    ) -> Result<(), String> {
        if self.variable_declared(identifier.clone()) {
            Err(String::from("Cannot redeclare a variable."))
        } else {
            self.variables.insert(identifier, (is_const, value));
            Ok(())
        }
    }

    pub fn variable_declared(&mut self, identifier: String) -> bool {
        self.resolve_variable_environment(&identifier).is_some()
    }

    pub fn resolve_variable_environment(&mut self, identifier: &String) -> Option<&mut Environment> {
        if self.variables.contains_key(identifier) {
            Some(self)
        } else {
            if let Some(parent) = &mut self.parent {
                parent.resolve_variable_environment(identifier)
            } else {
                None
            }
        }
    }

    pub fn get_variable(&mut self, identifier: String) -> Result<RuntimeValue, String> {
        if let Some(env) = self.resolve_variable_environment(&identifier) {
            Ok(env.variables.get(&identifier).unwrap().clone().1)
        } else {
            Err(String::from("Variable is not declared."))
        }
    }

    pub fn is_const(&self, identifier: String) -> bool {
        let (is_const, _) = self.variables.get(&identifier).unwrap();

        is_const.clone()
    }

    pub fn set_variable(&mut self, identifier: String, value: RuntimeValue) -> Result<(), String> {
        if let Some(mut env) = self.resolve_variable_environment(&identifier) {
            if env.is_const(identifier.clone()) {
                Err(String::from("Unable to reassign value to a constant"))
            } else {
                env.variables.insert(identifier, (false, value));
                Ok(())
            }
        } else {
            Err(String::from("Variable is not declared."))
        }
    }
}

// impl Environment {
//     pub(crate) fn default() -> &'static mut Self {
//         let mut env = Self::new();
//         // 
//         &mut env
//     }
// }