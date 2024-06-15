use std::collections::HashMap;
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::Command;
use std::{env, fs};

fn command_type(cmd: &Command) {
    let cmd_program = cmd.get_program().to_os_string();

    if cmd_program.to_ascii_lowercase().to_str().unwrap() == "type" {
        // Collect the arguments into a vector
        let args: Vec<OsString> = cmd.get_args().map(|arg| arg.to_os_string()).collect();
        let envs: Vec<(String, String)> = std::env::vars().collect();

        if let Some(new_cmd) = args.first() {
            let mut target_command = Command::new(new_cmd);
            target_command.current_dir(cmd.get_current_dir().unwrap());

            // Skip the first argument since it's the new command
            for arg in &args[1..] {
                target_command.arg(arg);
            }
            for (key, val) in envs {
                target_command.env(key, val);
            }

            let program = target_command.get_program().to_ascii_lowercase();

            match program.to_str() {
                Some(p) => match p {
                    "cd" | "pwd" | "echo" | "type" | "which" | "exit" | "export" => {
                        println!("{} is a shell builtin", p)
                    }
                    _ => {
                        let location = _find(&target_command);
                        if location.is_empty() {
                            println!("{}: not found", p);
                        } else {
                            println!("{} is {}", p, location)
                        }
                    }
                },
                None => println!("Unknown"),
            }
        }
    }
}

fn command_export(cmd: &Command) {
    let args: Vec<&str> = cmd.get_args().map(|arg| arg.to_str().unwrap()).collect();
    //    let envs: Vec<(String, String)> = std::env::vars().collect();
    if args.len() == 1 && args.first().unwrap().contains('=') {
        let parts: Vec<&str> = args.first().unwrap().split('=').collect();
        if parts.len() == 2 {
            std::env::set_var(parts[0], parts[1]);
        } else {
            println!("Error setting var");
        }
    } else {
        println!("Error: Bad format");
    }
}
fn command_pwd(cmd: &Command) {
    println!("{}", cmd.get_current_dir().unwrap().to_str().unwrap());
}
fn command_cd(cmd: &Command) {
    let args: Vec<&str> = cmd.get_args().map(|arg| arg.to_str().unwrap()).collect();

    if args.len() > 1 {
        println!("Error: Bad format");
        return;
    }

    let dir = *args.first().unwrap();

    if dir.starts_with('/') {
        // absolute path
        match env::set_current_dir(dir) {
            Ok(_) => (),
            Err(_) => println!("cd: {}: No such file or directory", dir),
            //Err(e) => println!("cd: {}: {}", dir, e),
        }
    }
}
fn command_env(cmd: &Command) {
    let envs: Vec<(String, String)> = cmd
        .get_envs()
        .filter_map(|(x, y)| {
            // Ensure both key and value are present and can be converted to String
            if let (Some(key), Some(value)) = (x.to_str(), y.as_ref().and_then(|v| v.to_str())) {
                Some((key.to_string(), value.to_string()))
            } else {
                None
            }
        })
        .collect();
    for (key, value) in envs {
        println!("{}={}", key, value);
    }
}
fn command_path(cmd: &Command) {
    if let Some((_, path_value)) = cmd
        .get_envs()
        .find(|(k, _)| k.to_ascii_lowercase() == "path")
    {
        if let Some(path_value) = path_value {
            if let Some(path_str) = path_value.to_str() {
                for path in path_str.split(':') {
                    println!("{}", path);
                }
            } else {
                eprintln!("Failed to convert PATH to string");
            }
        } else {
            eprintln!("PATH value is None");
        }
    } else {
        eprintln!("PATH not found in command environment variables");
    }
}

fn _find(cmd: &Command) -> String {
    let mut location = String::new();
    if let Some((_, path_value)) = cmd
        .get_envs()
        .find(|(k, _)| k.to_ascii_lowercase() == "path")
    {
        if let Some(path_value) = path_value {
            if let Some(path_str) = path_value.to_str() {
                for path in path_str.split(':') {
                    if let Ok(entries) = fs::read_dir(path) {
                        entries.for_each(|entry| {
                            if let Ok(entry) = entry {
                                if entry.path().file_name().unwrap() == cmd.get_program() {
                                    location = entry.path().to_str().unwrap().to_string();
                                }
                            }
                        });
                    }
                }
            } else {
                eprintln!("Failed to convert PATH to string");
            }
        } else {
            eprintln!("PATH value is None");
        }
    } else {
        eprintln!("PATH not found in command environment variables");
    }
    location
}

fn command_which(cmd: &Command) {
    if cmd.get_program().to_ascii_lowercase().to_str().unwrap() == "which" {
        // Collect the arguments into a vector
        let args: Vec<OsString> = cmd.get_args().map(|arg| arg.to_os_string()).collect();
        let envs: Vec<(String, String)> = std::env::vars().collect();

        if let Some(new_cmd) = args.first() {
            let mut target_command = Command::new(new_cmd);
            target_command.current_dir(cmd.get_current_dir().unwrap());

            // Skip the first argument since it's the new command
            for arg in &args[1..] {
                target_command.arg(arg);
            }
            for (key, val) in envs {
                target_command.env(key, val);
            }
            let location = _find(&target_command);
            if location.is_empty() {
                println!(
                    "{}: not found",
                    target_command.get_program().to_str().unwrap()
                );
            } else {
                println!(
                    "{} is {}",
                    target_command.get_program().to_str().unwrap(),
                    location
                );
            }
        }
    }
}
fn command_exit(cmd: &Command) {
    let args: Vec<&str> = cmd.get_args().map(|arg| arg.to_str().unwrap()).collect();
    let exit_code: i32 = args[0].parse::<i32>().unwrap();
    std::process::exit(exit_code);
}

fn command_echo(cmd: &Command) {
    let args: Vec<&str> = cmd.get_args().map(|arg| arg.to_str().unwrap()).collect();
    println!("{}", args.join(" "));
}

fn command_unknown(cmd: &Command) {
    println!("{}: command not found", cmd.get_program().to_str().unwrap());
}

fn command_exec(mut cmd: Command) {
    let output = cmd.output();

    match output {
        Ok(output) => {
            if output.status.success() {
                io::stdout().write_all(&output.stdout).unwrap();
            } else {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(_) => {
            command_unknown(&cmd);
        }
    }
}

fn process_input(command_string: String) {
    let parts: Vec<String> = command_string.split(' ').map(|s| s.to_string()).collect();
    let mut iter = parts.iter().peekable();
    let mut temp_vars: HashMap<String, String> = HashMap::new();

    let global_variables: HashMap<String, String> = std::env::vars().collect();
    let directory = std::env::current_dir().unwrap();

    if iter.peek().unwrap().contains('=') {
        //loop while top of parts iterator contains "="
        while iter.peek().unwrap().contains('=') {
            let part = iter.next().unwrap();
            let mut split = part.splitn(2, '=');
            let key = split.next().unwrap_or("").trim_matches('\"').to_string();
            let value = split.next().unwrap_or("").trim_matches('\"').to_string();
            temp_vars.insert(key, value);
        }
    }
    let temporary_vars = temp_vars.clone();
    let program = iter.next().unwrap();

    let mut cmd = Command::new(program);
    cmd.current_dir(directory);

    for part in iter {
        cmd.arg(part.clone());
    }

    for (k, v) in global_variables.iter() {
        cmd.env(k, v);
    }

    for (k, v) in temporary_vars.iter() {
        cmd.env(k, v);
    }

    match cmd.get_program().to_str().unwrap() {
        "which" => command_which(&cmd),
        "env" => command_env(&cmd),
        "pwd" => command_pwd(&cmd),
        "cd" => command_cd(&cmd),
        "path" => command_path(&cmd),
        "export" => {
            command_export(&cmd);
            let command_envs: Vec<(String, String)> = cmd
                .get_envs()
                .map(|(x, y)| {
                    (
                        x.to_str().unwrap().to_string(),
                        y.unwrap().to_str().unwrap().to_string(),
                    )
                })
                .collect();

            for (key, value) in command_envs {
                if !temporary_vars.contains_key(&key) && !global_variables.contains_key(&key) {
                    std::env::set_var(key, value);
                }
            }
        }
        "type" => command_type(&cmd),
        "echo" => command_echo(&cmd),
        "exit" => command_exit(&cmd),
        _ => command_exec(cmd),
    }
}

fn main() {
    let stdin = io::stdin();

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).expect("Failed to read line");
        let command_string = input.trim().to_string();
        process_input(command_string);
    }
}
