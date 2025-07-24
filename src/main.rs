use infra::shell::ShellCommandDetector;

use std::process::Command;
use std::env;
use std::path::Path;
use std::cell::RefCell;
use std::rc::Rc;

use rustyline::Context;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Editor, Helper};
use hostname::get;

struct ShellCommandHinter {
    detector: Rc<RefCell<ShellCommandDetector>>,
}

impl Hinter for ShellCommandHinter {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        let detector = self.detector.borrow();
        let suggestions = detector.get_command_suggestions(line);
        suggestions.first().and_then(|suggestion| {
            if suggestion.starts_with(line) {
                Some(suggestion[line.len()..].to_string()) // only the suffix as hint
            } else {
                Some(suggestion.clone())
            }
        })
    }
}

struct InputHelper {
    hinter: ShellCommandHinter,
}

impl Helper for InputHelper {}

impl Hinter for InputHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Completer for InputHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        _line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), rustyline::error::ReadlineError> {
        Ok((0, Vec::new()))
    }
}

impl Highlighter for InputHelper {
    // Dim the hint text (grey color)
    fn highlight_hint<'h>(&self, hint: &'h str) -> std::borrow::Cow<'h, str> {
        format!("\x1b[90m{}\x1b[0m", hint).into()
    }
}

impl Validator for InputHelper {
    fn validate(
        &self,
        _ctx: &mut ValidationContext,
    ) -> Result<ValidationResult, rustyline::error::ReadlineError> {
        Ok(ValidationResult::Valid(None))
    }
}

fn main() -> rustyline::Result<()> {
    let detector = Rc::new(RefCell::new(ShellCommandDetector::new()));

    let helper = InputHelper {
        hinter: ShellCommandHinter {
            detector: Rc::clone(&detector),
        },
    };

    let mut rl = Editor::new()?;
    rl.set_helper(Some(helper));

    println!("Enter commands to check (type 'exit' to quit):");

    let username = env::var("USER").unwrap_or_else(|_| "user".to_string());
    let hostname = get().unwrap_or_default().into_string().unwrap_or_default();

    loop {
        let current_dir = env::current_dir().unwrap_or_else(|_| ".".into());
        // ANSI color codes:
        // Red: \x1b[31m
        // Green: \x1b[32m
        // Yellow: \x1b[33m
        // Blue: \x1b[34m
        // Reset: \x1b[0m
        let prompt = format!("\x1b[32m{}@{}\x1b[0m:\x1b[34m{}\x1b[0m\n\x1b[31minfraware\x1b[0m$ ", username, hostname, current_dir.display());
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.eq_ignore_ascii_case("exit") {
                    break;
                }
                let is_shell_command = detector.borrow_mut().is_shell_command(input);

                println!("Is shell command: {}", is_shell_command);
                if !is_shell_command {
                    continue;
                }

                let mut parts = input.trim().split_whitespace();

                let command = match parts.next() {
                    Some(command) => command,
                    None => continue, // skip to next iteration if None
                };
                let args = parts;
                match command {
                    "cd" => {
                        let new_dir = args.peekable().peek().map_or("/", |x| *x);
                        let root = Path::new(new_dir);
                        if let Err(e) = env::set_current_dir(&root) {
                            eprintln!("CD error: {e}");
                        };
                        continue;
                    },
                    _ => {},
                }
                let mut child = match Command::new(command).args(args).spawn() {
                    Ok(child) => child,
                    Err(e) => {
                        eprintln!("Application error: {e}");
                        continue;
                    },
                };
                let _ = child.wait();
            }
            Err(_) => println!("No Input"),
        }
    }

    println!("Goodbye!");
    Ok(())
}
