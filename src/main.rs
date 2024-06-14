use std::fmt;
#[allow(unused_imports)]
use std::io::{self, Write};

struct Command<'a> {
    parts: Vec<&'a str>,
}

impl<'a> Command<'a> {
    fn new(command_string: &'a str) -> Self {
        Self {
            parts: command_string.trim().split(' ').collect(),
        }
    }

    fn executable(&self) -> &'a str {
        self.parts[0]
    }

    fn args(&self) -> Vec<&'a str> {
        let [_, args @ ..] = self.parts.as_slice() else {
            todo!()
        };
        args.to_vec()
    }
}

impl<'a> fmt::Display for Command<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.parts.join(" "))
    }
}

fn command_exit(cmd: Command) {
    let exit_code: i32 = cmd.args().join("").parse::<i32>().unwrap();
    std::process::exit(exit_code);
}
fn command_echo(cmd: Command) {
    println!("{}", cmd.args().join(" "));
}
fn command_unknown(cmd: Command) {
    println!("{}: command not found", cmd);
}

fn process_input(cmd: Command) {
    match cmd.executable() {
        "echo" => command_echo(cmd),
        "exit" => command_exit(cmd),
        _ => command_unknown(cmd),
    }
}

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        match stdin.read_line(&mut input) {
            Ok(_) => {
                let cmd = Command::new(&input);
                process_input(cmd);
                input = String::new();
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
