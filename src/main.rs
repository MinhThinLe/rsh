#![deny(clippy::complexity, clippy::pedantic, clippy::perf, clippy::nursery)]

mod commands;
mod input;
mod shell;
mod tokenizer;

use shell::Shell;

fn main() {
    let shell = Shell::start();
    shell.process();
}
