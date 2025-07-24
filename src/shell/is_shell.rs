use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

pub struct ShellCommandDetector {
    available_commands: HashSet<String>,
}

impl ShellCommandDetector {
    pub fn new() -> Self {
        let mut detector = ShellCommandDetector {
            available_commands: HashSet::new(),
        };
        detector.load_commands();
        detector
    }

    fn load_commands(&mut self) {
        if let Some(path_env) = env::var_os("PATH") {
            for dir in env::split_paths(&path_env) {
                self.load_commands_from_directory(&dir);
            }
        }
        self.load_user_aliases();
    }

    fn load_commands_from_directory(&mut self, dir: &Path) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && is_executable(&entry.path()) {
                        if let Some(name) = entry.file_name().to_str() {
                            self.available_commands.insert(name.to_string());
                        }
                    }
                }
            }
        }
    }

    fn load_user_aliases(&mut self) {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

        if let Ok(output) = Command::new(&shell).args(["-i", "-c", "alias"]).output() {
            if output.status.success() {
                let aliases = String::from_utf8_lossy(&output.stdout);
                for line in aliases.lines() {
                    if let Some(alias) = Self::parse_alias(line) {
                        self.available_commands.insert(alias);
                    }
                }
            }
        }
    }

    fn parse_alias(line: &str) -> Option<String> {
        static ALIAS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"alias\s+([^=]+)=").unwrap());

        ALIAS_RE
            .captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    fn is_obvious_natural_language(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();

        if text_lower.ends_with('?') {
            return true;
        }

        let question_words = ["what ", "how ", "why ", "when ", "where ", "who "];
        if question_words.iter().any(|w| text_lower.starts_with(w)) {
            return true;
        }

        let starters = [
            "tell me",
            "can you",
            "please ",
            "i want",
            "i need",
            "could you",
            "would you",
            "help me",
        ];
        starters.iter().any(|s| text_lower.starts_with(s))
    }

    fn is_valid_command(&mut self, command: &str) -> bool {
        // First check if it's a path
        if command.starts_with("./") || command.starts_with('/') || command.contains('/') {
            return true;
        }

        // Then check builtins
        let builtins: HashSet<&str> = [
            "ls", "cd", "pwd", "echo", "export", "source", "alias", "unalias", "history", "jobs",
            "fg", "bg", "kill", "wait", "exec", "eval", "test", "[", "printf", "read", "set",
            "unset", "shift", "exit", "return", "break", "continue", "which", "type", "command",
            "builtin", "declare", "local", "readonly", "true", "false", "git", "mkdir", "rm", "cp",
            "mv", "cat", "grep", "find", "chmod", "sudo", "apt", "yum", "dnf", "pacman", "brew",
            "docker", "ssh", "clear",
        ]
        .iter()
        .cloned()
        .collect();

        if builtins.contains(command) {
            return true;
        }

        // Then check available commands
        if self.available_commands.contains(command) {
            return true;
        }

        // Finally check runtime
        self.command_exists_runtime(command)
    }

    fn command_exists_runtime(&mut self, command: &str) -> bool {
        if let Ok(output) = Command::new("which").arg(command).output() {
            if output.status.success() {
                self.available_commands.insert(command.to_string());
                return true;
            }
        }
        false
    }

    pub fn is_shell_command(&mut self, user_input: &str) -> bool {
        if user_input.trim().is_empty() {
            return false;
        }

        if self.is_obvious_natural_language(user_input) {
            return false;
        }

        // Try to parse shell tokens
        if let Ok(tokens) = shell_words::split(user_input) {
            if tokens.is_empty() {
                return false;
            }

            let command = &tokens[0];
            let args = if tokens.len() > 1 { &tokens[1..] } else { &[] };

            if !self.is_valid_command(command) {
                return false;
            }

            // Check arguments for natural language patterns
            return self.args_follow_shell_patterns_lenient(user_input, args);
        }

        false
    }

    fn args_follow_shell_patterns_lenient(
        &self,
        original_input: &str,
        parsed_args: &[String],
    ) -> bool {
        if parsed_args.is_empty() {
            return true;
        }

        // Check if input contains quotes
        let has_quotes = original_input.contains('\'') || original_input.contains('"');

        if has_quotes {
            // For quoted input, only check unquoted parts
            let unquoted_parts = self.extract_unquoted_parts(original_input);
            if unquoted_parts.trim().is_empty() {
                return true;
            }
            self.check_natural_language_patterns(&unquoted_parts)
        } else {
            // No quotes, check all arguments
            let combined_args = parsed_args.join(" ");
            self.check_natural_language_patterns(&combined_args)
        }
    }

    fn extract_unquoted_parts(&self, text: &str) -> String {
        let mut result = String::new();
        let mut in_quote = false;
        let mut quote_char = None;

        for c in text.chars() {
            if !in_quote && (c == '\'' || c == '"') {
                in_quote = true;
                quote_char = Some(c);
            } else if in_quote && Some(c) == quote_char {
                in_quote = false;
                quote_char = None;
            } else if !in_quote {
                result.push(c);
            }
        }

        // Remove the command part (first word)
        let unquoted_text = result.trim();
        if let Some(first_space) = unquoted_text.find(' ') {
            unquoted_text[first_space..].trim().to_string()
        } else {
            String::new()
        }
    }

    fn check_natural_language_patterns(&self, text: &str) -> bool {
        if text.trim().is_empty() {
            return true;
        }

        let text_lower = text.to_lowercase();

        // Natural language indicators
        let natural_indicators = [
            // Comparative language
            r"\b(better|worse|best|worst)\s+(than|of)\b",
            r"\bcompared?\s+to\b",
            r"\bvs\b|\bversus\b",
            // Possessive and descriptive patterns
            r"\bmy\s+(favorite|preferred|personal)\b",
            r"\bis\s+my\s+(favorite|preferred)\b",
            r"\bcan\s+(locate|find|search|help|assist|manage|handle|create|remove|display|show)\b",
            r"\b(helps?|assists?)\s+(navigate|with|you|me|us)\b",
            // Question patterns
            r"\bis\s+(the|this|that|a|an)\b",
            r"\bare\s+(the|these|those)\b",
            r"\bwhich\s+(one|is|are)\b",
            // Conversational patterns
            r"\b(can|could|should|would)\s+you\b",
            r"\bplease\s+(help|tell|show)\b",
            r"\btell\s+me\s+about\b",
            r"\bhelp\s+me\s+(with|understand)\b",
            // Articles + descriptive words
            r"\bthe\s+(latest|newest|oldest|current|main|primary|best)\b",
            r"\ba\s+(new|good|bad|better|simple|useful)\b",
            r"\ban\s+(old|new|existing)\b",
            // Explanatory language
            r"\bhow\s+(to|do|does)\b",
            r"\bwhat\s+(is|are|does)\b",
            r"\bwhy\s+(is|are|does)\b",
            // Natural language instruction patterns
            r"\bthis\s+(command|file|directory|is)\b",
            r"\bthat\s+(command|file|directory)\b",
            r"\ball\s+\w+\s+(files|directories|commands|in)\b",
        ];

        for pattern in &natural_indicators {
            let re = Regex::new(pattern).unwrap();
            if re.is_match(&text_lower) {
                return false;
            }
        }

        // Additional heuristics
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        if words.len() > 2 {
            let common_words: HashSet<&str> = [
                "the", "is", "are", "and", "or", "but", "for", "to", "of", "in", "on", "at", "by",
                "with", "all",
            ]
            .iter()
            .cloned()
            .collect();

            let common_count = words.iter().filter(|&w| common_words.contains(w)).count();
            let common_ratio = common_count as f32 / words.len() as f32;
            if common_ratio > 0.4 {
                return false;
            }
        }

        true
    }

    pub fn get_command_suggestions(&self, partial: &str) -> Vec<String> {
        self.available_commands
            .iter()
            .filter(|cmd| cmd.starts_with(partial))
            .take(10)
            .cloned()
            .collect()
    }
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            metadata.permissions().mode() & 0o111 != 0
        } else {
            false
        }
    }

    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_shell_command() {
        let mut detector = ShellCommandDetector::new();

        // Shell commands
        assert!(detector.is_shell_command("ls -la"));
        assert!(detector.is_shell_command("git status"));
        assert!(detector.is_shell_command("./my_script.sh"));
        assert!(detector.is_shell_command("cd ~/projects"));
        assert!(detector.is_shell_command("echo 'hello world'"));
        assert!(detector.is_shell_command("sudo apt update"));
        assert!(detector.is_shell_command("docker ps -a"));

        // Natural language
        assert!(!detector.is_shell_command("what files are in this directory?"));
        assert!(!detector.is_shell_command("please show me the files"));
        assert!(!detector.is_shell_command("how do I list files?"));
        assert!(!detector.is_shell_command("can you help me find all text files"));
        assert!(!detector.is_shell_command("what is the best way to list directories"));

        // Commands with quoted arguments
        assert!(detector.is_shell_command("grep 'search pattern' file.txt"));
        assert!(detector.is_shell_command("echo \"hello world\""));
    }

    #[test]
    fn test_natural_language_detection() {
        let detector = ShellCommandDetector::new();

        // Natural language patterns
        assert!(!detector.check_natural_language_patterns("better than the other command"));
        assert!(!detector.check_natural_language_patterns("my favorite command is"));
        assert!(!detector.check_natural_language_patterns("can you help me with"));
        assert!(!detector.check_natural_language_patterns("the best way to do this"));

        // Shell-like patterns
        assert!(detector.check_natural_language_patterns("-la ~/Documents"));
        assert!(detector.check_natural_language_patterns("--all --long /path/to/dir"));
        assert!(detector.check_natural_language_patterns("file.txt *.log"));
    }

    #[test]
    fn test_extract_unquoted_parts() {
        let detector = ShellCommandDetector::new();

        assert_eq!(
            detector.extract_unquoted_parts("command 'quoted arg' unquoted"),
            "unquoted"
        );
        assert_eq!(
            detector.extract_unquoted_parts("command \"quoted arg\" unquoted"),
            "unquoted"
        );
        assert_eq!(detector.extract_unquoted_parts("command 'quoted arg'"), "");
        assert_eq!(
            detector.extract_unquoted_parts("command unquoted"),
            "unquoted"
        );
    }
}
