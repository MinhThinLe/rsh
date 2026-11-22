mod commands;
mod shell;
mod tokenizer;

use shell::Shell;

fn main() {
    let shell = Shell::start();
    shell.process();
}
