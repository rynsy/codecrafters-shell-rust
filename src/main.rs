#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        match stdin.read_line(&mut input) {
            Ok(_) => {
                input = input.trim().to_string();
                println!("{input}: command not found");
                input = String::new();
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
