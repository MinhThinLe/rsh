use std::io::{Write, stdout};
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender, channel};

use crate::commands::{ChildProcess, get_builtin_handle, spawn_child};
use crate::input::{Input, input_thread};
use crate::tokenizer::{TokenizingError, Tokens};

pub enum ShellEvent {
    CtrlC,
    Input(Input),
    NewChild(ChildProcess),
    ChildExited(u32),
}

pub struct Shell {
    events: Receiver<ShellEvent>,
    sender: Sender<ShellEvent>,
    foreground_process: Option<ChildProcess>,
    pub(crate) processess: Vec<u32>,
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
            processess: Vec::new(),
        }
    }

    pub fn process(mut self) {
        print_prompt();
        loop {
            let event = self.events.recv().expect("Unable to receive an event");
            match event {
                ShellEvent::CtrlC => self.handle_ctrl_c(),
                ShellEvent::Input(string) => self.process_input(string),
                ShellEvent::NewChild(child) => self.handle_new_child(child),
                ShellEvent::ChildExited(id) => self.handle_child_exit(id),
            }
        }
    }

    fn handle_ctrl_c(&mut self) {
        if self.foreground_process.is_none() {
            println!();
            print_prompt();
            return;
        }
        // Waits for the child process to terminate from Ctrl-C, extremely hacky
        std::thread::sleep(std::time::Duration::from_millis(2));
        println!()
    }

    fn handle_new_child(&mut self, child: ChildProcess) {
        self.processess.push(child.id());
        if child.is_foreground() {
            self.foreground_process = Some(child);
        } else {
            print_prompt();
        }
    }

    fn handle_child_exit(&mut self, child_id: u32) {
        self.processess.retain(|id| *id != child_id);
        if let Some(ref child) = self.foreground_process
            && child.id() == child_id
        {
            self.foreground_process = None;
        }
        print_prompt();
    }

    fn process_input(&mut self, input: Input) {
        if let Some(ref mut child) = self.foreground_process {
            child
                .handle_input(input)
                .expect("Failed while trying to give a child input");
            return;
        }
        let command = match input {
            Input::Newline => {
                print_prompt();
                return;
            }
            Input::Eof => {
                println!();
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
            builtin(self, tokens.into_args());
            print_prompt();
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
