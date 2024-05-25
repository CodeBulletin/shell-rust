#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;

enum Command {
    Exit(i32),
    Echo(Vec<String>),
    Type(String),
    CD(String),
    SHELL(),
    EXEC(String, Vec<String>),
    NotBuiltIn(String, Vec<String>),
}

impl Command {
    fn execute(&self) {
        match self {
            Command::Exit(code) => std::process::exit(*code),
            Command::Echo(args ) => {
                println!("{}", args.join(" "));
            },
            Command::Type(command) => {
                if command.is_empty() {
                    println!("type: not enough arguments");
                    return;
                }
                if is_builtin(command) {
                    println!("{} is a shell builtin", command);
                } else {
                    match search_path(command) {
                        Some(path) => println!("{} is {}", command, path.to_str().unwrap()),
                        None => println!("{} not found", command),
                    }
                }
            },
            Command::CD(dir) => {
                let mut directory = PathBuf::from(dir);
                if dir.is_empty() {
                    directory = std::env::home_dir().unwrap();
                }
                if directory.exists() {
                    std::env::set_current_dir(directory).expect("failed to change directory");
                } else {
                    println!("{}: No such file or directory", dir);
                }
            },
            Command::SHELL() => {
                let shell_path = shell_path();
                println!("{}", shell_path.to_str().unwrap());
            },
            Command::EXEC(file, args) => {
                // load the file
                let path = PathBuf::from(file);
                if path.exists() {
                    // check if the file is executable
                    let permissions = std::fs::metadata(&path).unwrap().permissions();
                    if !permissions.mode() & 0o111 != 0 {
                        println!("{}: Permission denied", file);
                        return;
                    }
                    let file_contents = std::fs::read_to_string(path).expect("failed to read file");
                    let lines = file_contents.lines();
                    for line in lines {
                        let command = parse_line(line.to_string());
                        // if the command is exit then break
                        if let Command::Exit(_) = command {
                            break;
                        }
                        command.execute();
                    }
                } else {
                    println!("{}: No such file or directory", file);
                }
            },
            Command::NotBuiltIn(command, args) => {
                if let Some(path) = search_path(command) {
                    let mut child = std::process::Command::new(path)
                        .args(args)
                        .stdout(std::io::stdout())
                        .stderr(std::io::stderr())
                        .spawn()
                        .expect("failed to execute child");
                    child.wait().expect("failed to wait on child");
                } else {
                    println!("{}: command not found", command);
                }
            },
        }
    }
    
}

fn parse_line(line: String) -> Command {
    let args = line
        .trim()
        .split(' ')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();

    let command = args[0].to_string();

    let command = match command.as_str() {
        "exit" => Command::Exit(0),
        "echo" => Command::Echo(args[1..].to_vec()),
        "type" => Command::Type(args.get(1).unwrap_or(&"".to_string()).clone()),
        "cd" => Command::CD(args.get(1).unwrap_or(&"".to_string()).clone()),
        "shell" => Command::SHELL(),
        "exec" => Command::EXEC(args.get(1).unwrap_or(&"".to_string()).clone(), if args.len() > 2 { args[2..].to_vec() } else { vec![] }),
        _ => Command::NotBuiltIn(command, args[1..].to_vec()),
    };
    command
}

fn is_builtin(command: &str) -> bool {
    match command {
        "exit" | "echo" | "type" | "cd" | "shell" | "exec" => true,
        _ => false,
    }
}

fn search_path(command: &str) -> Option<PathBuf> {
    let path = std::env::var("PATH").unwrap();
    let paths: Vec<_> = path
        .split(":")
        .map(|p| PathBuf::from(p))
        .collect::<Vec<PathBuf>>();
    for bin_dir in paths {
        let full_path = bin_dir.join(command);
        if full_path.exists() {
            return Some(full_path);
        }
    }
    None
}

fn shell_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path
}

fn current_path() -> PathBuf {
    std::env::current_dir().unwrap()
}

fn main() {
    loop {
        let current_path = current_path();
        print!("{} $ ", current_path.to_str().unwrap());
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let command = parse_line(input);
        command.execute();
        io::stdout().flush().unwrap();
    }
}
