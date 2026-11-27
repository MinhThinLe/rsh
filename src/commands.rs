use std::io::{PipeWriter, Write};
use std::process::{Command, Stdio, exit};
use std::sync::mpsc::Sender;

use crate::input::Input;
use crate::platforms::get_process_name;
use crate::shell::{Shell, ShellEvent};
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

    pub const fn is_foreground(&self) -> bool {
        !self.is_background
    }

    pub const fn id(&self) -> u32 {
        self.id
    }

    pub fn handle_input(&mut self, input: Input) -> std::io::Result<()> {
        let Some(ref mut stdin) = self.stdin else {
            return Ok(());
        };

        match input {
            Input::Eof => {
                self.close_stdin();
                Ok(())
            }
            Input::Newline => stdin.write_all(b"\n"),
            Input::Line(line) => stdin.write_all(line.as_bytes()),
        }
    }
}

pub fn get_builtin_handle(command: &str) -> Option<Box<dyn Fn(&Shell, Args)>> {
    match command {
        "exit" => Some(Box::new(exit_handle)),
        "list" => Some(Box::new(list_processes_handle)),
        _ => None,
    }
}

fn exit_handle(_shell: &Shell, _args: Args) {
    exit(0)
}

fn list_processes_handle(shell: &Shell, _args: Args) {
    println!("processess");
    for process in shell.processess.iter() {
        let process_name = get_process_name(*process);
        println!("\t{process}\t{process_name}");
    }
}

pub fn spawn_child(command: Tokens, sender: Sender<ShellEvent>) {
    let is_background = command.is_background_process();
    let (mut command, stdin) = create_command(command);

    let mut process = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    let child = ChildProcess {
        stdin,
        id: process.id(),
        is_background,
    };

    std::thread::spawn(move || {
        sender
            .send(ShellEvent::NewChild(child))
            .expect("Unable to send an event");
        process.wait().expect("Err lol");
        sender
            .send(ShellEvent::ChildExited(process.id()))
            .expect("Unable to send an event");
    });
}

fn create_command(tokens: Tokens) -> (Command, Option<PipeWriter>) {
    let is_background = tokens.is_background_process();
    let (reader, writer) = std::io::pipe().expect("Failed to create a new pipe");
    let mut command = Command::new(tokens.command());

    command.stdin(if is_background {
        Stdio::null()
    } else {
        reader.into()
    });
    command.stdout(get_io_source(is_background));
    command.stderr(get_io_source(is_background));
    command.args(tokens.into_args().0);

    (command, if is_background { None } else { Some(writer) })
}

fn get_io_source(is_background: bool) -> Stdio {
    if is_background {
        Stdio::null()
    } else {
        Stdio::inherit()
    }
}
