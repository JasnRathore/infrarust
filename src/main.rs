use infra::shell::ShellCommandDetector;
use infra::shell::ShellCommandExecutor;
use rustyline::completion::Candidate;

use std::cell::RefCell;
use std::env;
use std::rc::Rc;

use hostname::get;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Editor, Helper};

struct ShellCommandHinter {
    detector: Rc<RefCell<ShellCommandDetector>>,
}

impl Hinter for ShellCommandHinter {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if line.is_empty() {
            return None; // no hint when input is empty
        }
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

fn print_help() {
    println!("Commands:");
    println!("    exit    'to quit'");
    println!("    help    'to print commands'");
    println!("Key Shorcuts:");
    println!("    Ctrl+l    'to print commands'");
}

fn main() -> rustyline::Result<()> {
    let detector = Rc::new(RefCell::new(ShellCommandDetector::new()));
    let mut executor = ShellCommandExecutor::new();

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
        let current_dir = executor.get_current_dir();
        // ANSI color codes:
        // Red: \x1b[31m
        // Green: \x1b[32m
        // Yellow: \x1b[33m
        // Blue: \x1b[34m
        // Reset: \x1b[0m
        let prompt = format!(
            "\x1b[32m{}@{}\x1b[0m:\x1b[34m{}\x1b[0m\n\x1b[31minfraware\x1b[0m$ ",
            username,
            hostname,
            current_dir.display()
        );

        let readline = match rl.readline(&prompt) {
            Ok(line) => line,
            Err(_) => {
                println!("No Input");
                continue;
            }
        };

        let input = readline.trim();
        match input.to_ascii_lowercase().as_str() {
            "" => {
                println!();
                continue;
            }
            "exit" => break,
            "help" => print_help(),
            _ => {}
        }

        let is_shell_command = detector.borrow_mut().is_shell_command(input);

        if is_shell_command {
            println!("\n\x1b[32mIt Is a shell command; \x1b[0m\n");
            let _ = rl.add_history_entry(input);
            executor.execute_shell_command(input.to_string());
        } else {
            println!("\n\x1b[31mIt Is not a shell command; \x1b[0m\n");
        }
    }

    println!("Goodbye!");
    Ok(())
}
