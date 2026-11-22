use std::process::{Command, exit};
use std::sync::mpsc::Sender;

use crate::shell::ShellEvent;
use crate::tokenizer::{Args, Tokens};

pub fn get_builtin_handle(command: &str) -> Option<Box<dyn Fn(Args)>> {
    match command {
        "exit" => Some(Box::new(exit_handle)),
        _ => None,
    }
}

fn exit_handle(_args: Args) {
    exit(0)
}

pub fn spawn_child(command: Tokens, sender: Sender<ShellEvent>) {
    let mut child = match Command::new(command.command())
        .args(command.into_args().0)
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    let child_pid = child.id();

    std::thread::spawn(move || {
        sender
            .send(ShellEvent::NewChild(child_pid))
            .expect("Unable to send an event");
        child.wait().expect("Err lol");
        sender
            .send(ShellEvent::ChildExited(child_pid))
            .expect("Unable to send an event");
    });
}
