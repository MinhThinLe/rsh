use std::process::exit;

use crate::tokenizer::Args;

pub fn get_builtin_handle(command: &str) -> Option<Box<dyn Fn(Args)>> {
    match command {
        "exit" => Some(Box::new(exit_handle)),
        _ => None,
    }
}

fn exit_handle(_args: Args) {
    exit(0)
}
