use crate::shell::Shell;

mod shell;

fn main() {
    let shell = Shell::start();
    shell.process();
}
