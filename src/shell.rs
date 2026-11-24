use std::io::{PipeWriter, Write, stdout};
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::commands::{get_builtin_handle, spawn_child};
use crate::input::{Input, input_thread};
use crate::tokenizer::{TokenizingError, Tokens};

pub enum ShellEvent {
    CtrlC,
    Input(Input),
    NewChild(u32, bool, PipeWriter),
    ChildExited(u32),
}

pub struct Shell {
    events: Receiver<ShellEvent>,
    sender: Sender<ShellEvent>,
    foreground_process: Option<(PipeWriter, u32)>,
}

impl Shell {
    pub fn start() -> Self {
        let (events_sender, events_receiver) = channel();

        ctrl_c_thread(events_sender.clone());
        input_thread(events_sender.clone());

        Self {
            events: events_receiver,
            sender: events_sender,
            foreground_process: None,
        }
    }

    pub fn process(mut self) {
        print_prompt();
        loop {
            let event = self.events.recv().expect("Unable to receive an event");
            match event {
                ShellEvent::CtrlC => {
                    println!("Ctrl c received");
                    print_prompt();
                }
                ShellEvent::Input(string) => self.process_input(string),
                ShellEvent::NewChild(id, is_background, stdin) => {
                    println!("Child process {id} spawned");
                    if !is_background {
                        self.foreground_process = Some((stdin, id))
                    }
                }
                ShellEvent::ChildExited(id) => {
                    println!("Child process {id} exited");
                    if let Some((ref _stdin, child_id)) = self.foreground_process
                        && id == child_id
                    {
                        self.foreground_process = None
                    }
                    print_prompt();
                }
            }
        }
    }

    fn process_input(&mut self, input: Input) {
        if let Some((ref mut stdin, _id)) = self.foreground_process {
            let input = match input {
                Input::Newline => vec![b'\n'],
                Input::EOF => {
                    self.foreground_process = None; // Properly drop this later
                    return;
                }
                Input::Line(line) => line.into_bytes(),
            };
            stdin.write(&input).expect("Unable to write to a pipe");
            stdin.flush().expect("Unable to flush stdin");
            return;
        }
        let command = match input {
            Input::Newline | Input::EOF => {
                print_prompt();
                return;
            }
            Input::Line(line) => line,
        };
        let tokens = match Tokens::from_str(&command) {
            Ok(token) => token,
            Err(TokenizingError::EmptyInput) => return, // Should not be possible
            Err(TokenizingError::UnmatchedQuote) => {
                println!("Unmatched \"");
                return;
            }
        };

        if let Some(builtin) = get_builtin_handle(tokens.command()) {
            builtin(tokens.into_args());
            return;
        }

        spawn_child(tokens, self.sender.clone());
    }
}

fn print_prompt() {
    const SHELL_PROMPT: &str = "-> ";
    stdout()
        .write_all(SHELL_PROMPT.as_bytes())
        .expect("Unable to write to stdout");
    stdout().flush().expect("Unable to flush stdout");
}

fn ctrl_c_thread(sender: Sender<ShellEvent>) {
    ctrlc::set_handler(move || {
        sender
            .send(ShellEvent::CtrlC)
            .expect("Unable to send an event");
    })
    .expect("Unable to set a ctrl-c handler");
}
