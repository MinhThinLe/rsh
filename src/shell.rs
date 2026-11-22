use std::io::{Write, stdout};
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::commands::get_builtin_handle;
use crate::tokenizer::{TokenizingError, Tokens};

enum ShellEvent {
    CtrlC,
    Input(String),
}

pub struct Shell {
    events: Receiver<ShellEvent>,
    sender: Sender<ShellEvent>,
}

impl Shell {
    pub fn start() -> Self {
        let (events_sender, events_receiver) = channel();

        ctrl_c_thread(events_sender.clone());
        input_thread(events_sender.clone());

        Shell {
            events: events_receiver,
            sender: events_sender,
        }
    }

    pub fn process(self) {
        loop {
            self.print_prompt();
            let event = self.events.recv().expect("Unable to receive an event");
            match event {
                ShellEvent::CtrlC => println!("Ctrl c received"),
                ShellEvent::Input(string) => self.process_command(string),
            }
        }
    }

    fn process_command(&self, command: String) {
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

        println!("Processing command: {command}")
    }

    fn print_prompt(&self) {
        const SHELL_PROMPT: &str = "-> ";
        stdout()
            .write_all(SHELL_PROMPT.as_bytes())
            .expect("Unable to write to stdout");
        stdout().flush().expect("Unable to flush stdout")
    }
}

fn ctrl_c_thread(sender: Sender<ShellEvent>) {
    ctrlc::set_handler(move || {
        sender
            .send(ShellEvent::CtrlC)
            .expect("Unable to send an event")
    })
    .expect("Unable to set a ctrl-c handler")
}

fn input_thread(sender: Sender<ShellEvent>) {
    std::thread::spawn(move || {
        let mut line = String::new();
        loop {
            std::io::stdin()
                .read_line(&mut line)
                .expect("Failed to read from stdin");

            let input = line.trim();
            if input.is_empty() {
                continue;
            }

            sender
                .send(ShellEvent::Input(input.to_string()))
                .expect("Failed to send an event");
            line.clear();
        }
    });
}
