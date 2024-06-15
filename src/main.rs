use std::fmt;
#[allow(unused_imports)]
use std::io::{self, Write};

enum CommandType {
    Builtin,
    //    Environment,
    Unknown,
}

struct Command<'a> {
    parts: Vec<&'a str>,
    command_type: CommandType,
}

impl<'a> Command<'a> {
    fn new(command_string: &'a str) -> Self {
        let parts: Vec<&str> = command_string.trim().split(' ').collect();
        let command_type: CommandType = match parts[0] {
            "exit" => CommandType::Builtin,
            "echo" => CommandType::Builtin,
            "type" => CommandType::Builtin,
            _ => CommandType::Unknown,
        };
        Self {
            parts,
            command_type,
        }
    }
    fn executable(&self) -> &'a str {
        self.parts[0]
    }
    fn args(&self) -> Vec<&'a str> {
        self.parts[1..].to_vec()
    }
    fn command_type(&self) -> &CommandType {
        &self.command_type
    }
}

impl<'a> fmt::Display for Command<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.parts.join(" "))
    }
}

fn command_type(cmd: Command) {
    match cmd.command_type() {
        CommandType::Builtin => println!("{} is a shell builtin", cmd.args()[0]),
        //       CommandType::Environment => println!("{} is at {}", cmd.executable(), cmd.executable()),
        CommandType::Unknown => println!("{}: not found", cmd.args()[0]),
    }
}
// fn command_which(cmd: Command) {
//     println!("{}", cmd)
// }
fn command_exit(cmd: Command) {
    let exit_code: i32 = cmd.args().join("").parse::<i32>().unwrap();
    std::process::exit(exit_code);
}
fn command_echo(cmd: Command) {
    println!("{}", cmd.args().join(" "));
}
fn command_unknown(cmd: Command) {
    println!("{}: command not found", cmd.executable());
}

fn process_input(cmd: Command) {
    match cmd.executable() {
        "type" => command_type(cmd),
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
