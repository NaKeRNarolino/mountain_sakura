use crate::interpreter::structs::RuntimeValue;
use crate::wrapper::bindings::MoSaNativeGen;
use crate::wrapper::MoSaRunner;

pub mod global;
pub mod interpreter;
pub mod lexer;
pub mod logging;
pub mod modules;
pub mod mosa_fs;
pub mod opts;
pub mod parser;
pub mod wrapper;

fn main() {
    let runner = MoSaRunner::new("./input/main.mosa")
        .add_lib("std", "./lib/std")
        .add_bindings(
            vec![
                |args: Vec<RuntimeValue>| -> RuntimeValue {
                    println!("{}", args[0]);
                    RuntimeValue::Null
                }.binding("mosa-native~>printLn"),
                |args: Vec<RuntimeValue>| -> RuntimeValue {
                    print!("{}", args[0]);
                    RuntimeValue::Null
                }.binding("mosa-native~>print")
            ],
        );

    dbg!(runner.run().unwrap());
}
