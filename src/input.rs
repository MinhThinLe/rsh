use std::sync::mpsc::Sender;

use crate::shell::ShellEvent;

pub enum Input {
    Newline,
    EOF,
    Line(String),
}

pub fn input_thread(sender: Sender<ShellEvent>) {
    std::thread::spawn(move || {
        let mut line = String::new();
        loop {
            std::io::stdin()
                .read_line(&mut line)
                .expect("Failed to read from stdin");

            let event = match line {
                ref _empty if line.is_empty() => Input::EOF,
                ref _newline if line.trim().is_empty() => Input::Newline,
                ref line => Input::Line(line.to_string()),
            };

            sender
                .send(ShellEvent::Input(event))
                .expect("Failed to send an event");

            line.clear();
        }
    });
}
