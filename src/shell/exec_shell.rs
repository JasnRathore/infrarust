use std::env;
use std::path::Path;
use std::process::Command;
use std::str::SplitWhitespace;

pub struct ShellCommandExecutor {
    current_dir: String,
    shell_history: Vec<String>,
}

impl ShellCommandExecutor {
    pub fn new() -> Self {
        let mut executor = ShellCommandExecutor {
            current_dir: String::new(),
            shell_history: Vec::new(),
        };
        executor.get_current_dir();
        executor
    }

    pub fn get_current_dir(&mut self) -> String {
        let current_dir = env::current_dir().unwrap_or_else(|_| ".".into());
        match current_dir.to_str() {
            Some(dir) => {
                self.current_dir = dir.to_string();
                return dir.to_string();
            }
            None => return String::new(),
        };
    }

    pub fn execute_shell_command(&mut self, input: String) {
        let mut parts = input.trim().split_whitespace();

        let command = match parts.next() {
            Some(command) => command,
            None => return, // skip to next iteration if None
        };
        let args = parts;
        match command {
            "cd" => {
                self.handle_cd_command(args);
                self.shell_history.push(input);
                return;
            }
            _ => {}
        }
        let mut child = match Command::new(command).args(args).spawn() {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Application error: {e}");
                return;
            }
        };
        self.shell_history.push(input);
        let _ = child.wait();
    }

    pub fn get_shell_history(&self) -> Vec<String> {
        self.shell_history.clone()
    }

    fn handle_cd_command(&self, args: SplitWhitespace<'_>) {
        let new_dir = args.peekable().peek().map_or("/", |x| *x);
        let root = Path::new(new_dir);
        if let Err(e) = env::set_current_dir(&root) {
            eprintln!("CD error: {e}");
        };
    }
}
