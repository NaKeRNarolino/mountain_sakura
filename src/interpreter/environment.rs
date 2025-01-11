use std::cell::RefCell;
use crate::interpreter::structs::RuntimeValue;
use std::collections::HashMap;
use std::process::id;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;
use crate::lexer::structs::KeywordType::Has;
use crate::parser::structs::ASTNode;

pub type FnArgs = HashMap<String, String>;

#[derive(Clone)]
pub struct EnvironmentMap {
    map: HashMap<Uuid, Rc<RefCell<Environment>>>,
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub variables: HashMap<String, (bool, RuntimeValue)>,
    pub functions: HashMap<String, (FnArgs, Vec<ASTNode>)>,
}

impl Environment {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            parent_id: None,
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn resolve_variable_environment<'a>(&self, variable_id: String, env_map: &'a EnvironmentMap) -> Option<&'a Rc<RefCell<Environment>>> {
        if self.variables.contains_key(&variable_id) {
            Some(env_map.get_environment_box(self.id))
        } else {
            if let Some(parent_id) = self.parent_id {
                Some(env_map.get_environment_box(parent_id).borrow_mut().resolve_variable_environment(variable_id, env_map)?)
            } else {
                None
            }
        }
    }

    pub fn resolve_variable(&mut self, variable_id: String, environment_map: &EnvironmentMap) -> (bool, RuntimeValue) {
        self.resolve_variable_environment(variable_id.clone(), environment_map).unwrap().borrow_mut().variables.get(&variable_id).cloned().expect("Variable cannot be found")
    }

    pub fn assign_variable(&mut self, variable_id: String, environment_map: &EnvironmentMap, value: RuntimeValue) {
        if self.resolve_variable_environment(variable_id.clone(), environment_map).is_some() {
            let var =self.resolve_variable(variable_id.clone(), environment_map);
            if var.0 == true {
                panic!("Cannot reassign a constant value.");
            } else {
                self.resolve_variable_environment(variable_id.clone(), environment_map).unwrap().borrow_mut().variables.insert(variable_id.clone(), (
                    var.0, value));
            }
        }
    }

    pub fn declare_variable(&mut self, variable_id: String, environment_map: &EnvironmentMap, value: RuntimeValue, is_const: bool) {
        self.variables.insert(variable_id, (is_const, value));
    }
}

impl EnvironmentMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn new_environment(&mut self, parent: Option<Uuid>) -> Uuid {
        let uuid = Uuid::new_v4();
        let mut env = Environment::new(uuid);
        env.parent_id = parent;

        self.map.insert(uuid, Rc::new(RefCell::new(env)));

        uuid
    }
    // pub fn get_environment_mut(&mut self, uuid: Uuid) -> Option<&mut Environment> {
    //     let env_box = self.map.get_mut(&uuid).expect("Environment not found.");
    //
    //     Some(&mut *env_box)
    // }
    //
    // pub fn get_environment(&self, uuid: Uuid) -> Option<&Environment> {
    //     let env_box = self.map.get(&uuid).expect("Environment not found.");
    //
    //     Some(&*env_box)
    // }

    pub fn get_environment_box(&self, uuid: Uuid) -> &Rc<RefCell<Environment>> {
        let env_box = self.map.get(&uuid).expect("Environment not found.");

        env_box
    }
}