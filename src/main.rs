use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Write};
use std::rc::Rc;

#[derive(Clone, Debug)]
struct Environment {
    path: Rc<RefCell<Vec<String>>>,
    variables: Rc<RefCell<HashMap<String, String>>>,
}

impl Environment {
    fn new() -> Self {
        Self {
            path: Rc::new(RefCell::new(Vec::new())),
            variables: Rc::new(RefCell::new({
                let mut vars = HashMap::new();
                vars.insert("DEFAULT_KEY".to_string(), "default_value".to_string());
                vars
            })),
        }
    }

    fn find(&self, cmd: &str) -> Option<String> {
        self.variables.borrow().get(cmd).cloned()
    }

    fn insert_var(&self, key: &str, val: String) {
        if key.to_lowercase() == "path" {
            let paths: Vec<String> = val.split(':').map(|s| s.to_string()).collect();
            for path in paths {
                self.insert_path(path);
            }
        }
        self.variables.borrow_mut().insert(key.to_string(), val);
    }

    fn insert_path(&self, val: String) {
        self.path.borrow_mut().push(val);
    }
}

enum CommandType {
    Builtin,
    Environment,
    Unknown,
}

#[derive(Clone, Debug)]
struct Command {
    parts: Vec<String>,
    env: Rc<Environment>,
}

impl Command {
    fn new(environment: Rc<Environment>) -> Self {
        Self {
            parts: Vec::new(),
            env: environment,
        }
    }

    fn with_parts(mut self, parts: Vec<String>) -> Self {
        self.parts = parts;
        self
    }

    fn from(command_string: String, env: Rc<Environment>) -> Self {
        let parts = command_string
            .split_whitespace()
            .map(String::from)
            .collect();
        Command::new(env).with_parts(parts)
    }

    fn executable(&self) -> &str {
        &self.parts[0]
    }

    fn args(&self) -> Vec<&str> {
        self.parts[1..].iter().map(AsRef::as_ref).collect()
    }

    fn arg_string(&self) -> String {
        self.args().join(" ")
    }

    fn command_type(&self) -> CommandType {
        match self.parts[0].as_str() {
            "export" | "env" | "exit" | "echo" | "type" => CommandType::Builtin,
            _ => CommandType::Unknown,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.parts.join(" "))
    }
}

fn command_type(cmd: &Command) {
    let binding = cmd.arg_string();
    let envbind = Rc::clone(&cmd.env);
    let target_command = Command::from(binding, envbind);
    match target_command.command_type() {
        CommandType::Builtin => println!("{} is a shell builtin", target_command.executable()),
        CommandType::Environment => {
            if let Some(location) = cmd.env.find(cmd.executable()) {
                println!("{} is at {}", cmd.executable(), location);
            }
        }
        CommandType::Unknown => println!("{}: not found", target_command.executable()),
    }
}

fn command_export(cmd: &Command) {
    if cmd.args().len() == 1 && cmd.args()[0].contains('=') {
        let parts: Vec<&str> = cmd.args()[0].split('=').collect();
        if parts.len() == 2 {
            cmd.env.insert_var(parts[0], parts[1].to_string());
        } else {
            println!("Error setting var");
        }
    } else {
        println!("Error: Bad format");
    }
}

fn command_env(cmd: &Command) {
    for (key, value) in cmd.env.variables.borrow().iter() {
        println!("{}={}", key, value);
    }
}

fn command_exit(cmd: &Command) {
    let exit_code: i32 = cmd.args().join("").parse().unwrap_or(0);
    std::process::exit(exit_code);
}

fn command_echo(cmd: &Command) {
    println!("{}", cmd.args().join(" "));
}

fn command_unknown(cmd: &Command) {
    println!("{}: command not found", cmd.executable());
}

fn process_input(cmd: Command) -> Rc<Environment> {
    let global_env = Rc::clone(&cmd.env);

    let local_env = Rc::new(Environment::new());
    let temp_env = Rc::new(Environment::new());

    for (key, value) in global_env.variables.borrow().iter() {
        local_env.insert_var(key, value.to_string());
    }

    let mut cmd = Command::new(local_env.clone()).with_parts(cmd.parts.clone());

    if cmd.executable().contains('=') {
        cmd = parse_environment_variables(&cmd);
        for (key, value) in cmd.env.variables.borrow().iter() {
            temp_env.insert_var(key, value.to_string());
        }
    }

    match cmd.executable() {
        "env" => command_env(&cmd),
        "export" => {
            command_export(&cmd);

            let output_env: Rc<Environment> = Rc::clone(&global_env);

            cmd.env.variables.borrow().iter().for_each(|(key, value)| {
                if !temp_env.variables.borrow().keys().any(|x| x == key)
                    && !global_env.variables.borrow().keys().any(|x| x == key)
                {
                    output_env.insert_var(key, value.to_string());
                }
            });
            // Need all keys that are not in the temp_env Environment
            return Rc::clone(&output_env);
        }
        "type" => command_type(&cmd),
        "echo" => command_echo(&cmd),
        "exit" => command_exit(&cmd),
        _ => command_unknown(&cmd),
    }

    global_env
}

fn parse_environment_variables(cmd: &Command) -> Command {
    let mut cmd = cmd.clone();
    while cmd.executable().contains('=') {
        let parts: Vec<&str> = cmd.executable().split('=').collect();
        let key = parts[0];
        let val = parts[1..].join("");
        let environment = Rc::clone(&cmd.env);
        environment.insert_var(key, val);
        cmd = Command::new(environment).with_parts(cmd.parts.clone()[1..].to_vec());
    }
    cmd
}

fn main() {
    let stdin = io::stdin();
    let mut environment = Rc::new(Environment::new());
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).expect("Failed to read line");

        let command_string = input.trim().to_string();
        let cmd = Command::from(command_string, Rc::clone(&environment));

        environment = process_input(cmd);
    }
}
