use std::io::{PipeWriter, Write};
use std::process::{Command, exit};
use std::sync::mpsc::Sender;

use crate::input::Input;
use crate::shell::ShellEvent;
use crate::tokenizer::{Args, Tokens};

pub struct ChildProcess {
    id: u32,
    stdin: Option<PipeWriter>,
    is_background: bool,
}

impl ChildProcess {
    fn close_stdin(&mut self) {
        self.stdin = None;
    }

    pub fn is_foreground(&self) -> bool {
        !self.is_background
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn handle_input(&mut self, input: Input) -> std::io::Result<()> {
        let Some(ref mut stdin) = self.stdin else {
            return Ok(());
        };

        match input {
            Input::Eof => {
                self.close_stdin();
                return Ok(());
            }
            Input::Newline => stdin.write_all(b"\n"),
            Input::Line(line) => stdin.write_all(line.as_bytes()),
        }
    }
}

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
    let is_background = command.is_background_process();
    let (reader, writer) = std::io::pipe().expect("Failed to create a new pipe");
    let mut process = match Command::new(command.command())
        .args(command.into_args().0)
        .stdin(reader)
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    let id = process.id();
    let child = ChildProcess {
        id,
        stdin: Some(writer),
        is_background,
    };

    std::thread::spawn(move || {
        sender
            .send(ShellEvent::NewChild(child))
            .expect("Unable to send an event");
        process.wait().expect("Err lol");
        sender
            .send(ShellEvent::ChildExited(id))
            .expect("Unable to send an event");
    });
}
