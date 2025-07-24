use infra::shell::ShellCommandDetector;

use rustyline::Context;
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Editor, Helper};

use std::cell::RefCell;
use std::rc::Rc;

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

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input.eq_ignore_ascii_case("exit") {
                    break;
                }
                let is_shell_command = detector.borrow_mut().is_shell_command(input);
                println!("Is shell command: {}", is_shell_command);
            }
            Err(_) => println!("No Input"),
        }
    }

    println!("Goodbye!");
    Ok(())
}
