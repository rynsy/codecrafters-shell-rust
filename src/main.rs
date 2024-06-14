#[allow(unused_imports)]
use std::io::{self, Write};

fn process_input(cmd: String) {
    let parts: Vec<&str> = cmd.split(' ').collect();
    if let ["exit", exit_code] = parts.as_slice() {
        std::process::exit(exit_code.parse::<i32>().unwrap());
    } else if let ["echo", args @ ..] = parts.as_slice() {
        println!("{}", args.join(" "));
    } else {
        println!("{}: command not found", parts[0]);
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
                let cmd = input.trim().to_string().clone();
                process_input(cmd);
                input = String::new();
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
