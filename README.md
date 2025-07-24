# InfraRust
This is a basic start to the porting of infraware cli from python to rust 
this port currently handles the shell detection system.

//Only unix systems properly supported for all features
## Current Capabilites
- **Dynamic Command Discovery**: Scans your system's PATH and user aliases
- **Natural Language Detection**: Recognizes conversational patterns and question structures  
- **Context-Aware Analysis**: Handles quoted arguments and complex command structures
- **Real-time Validation**: Performs runtime checks for command availability

## Project Requirements

- **Rust**: Edition 2024 or later
- **Operating System**: Unix-like systems (Linux, macOS) or Windows
- **Shell Access**: For alias detection and command validation

## Dependencies
- **rustyline** (16.0.0): Powers the interactive readline interface with command hints
- **regex** (1.11.1): Handles pattern matching for natural language detection
- **shell-words** (1.1.0): Properly parses shell command syntax
- **once_cell** (1.21.3): Provides thread-safe lazy static initialization

## Getting Started

### Building the Project

Compile and run the interactive demo:

```bash
cargo run
```

### Using as a Library

Add InfraRust to your project's dependencies in `Cargo.toml`:

```toml
[dependencies]
infra = { path = "path/to/infrarust" }
```

Then import and use the shell command detector:

```rust
use infra::shell::ShellCommandDetector;

let mut detector = ShellCommandDetector::new();

// Check if input is a shell command
let is_command = detector.is_shell_command("ls -la");
println!("Is shell command: {}", is_command); // true

let is_natural = detector.is_shell_command("what files are here?");
println!("Is shell command: {}", is_natural); // false
```

## How to Run the Application

The interactive demo provides a hands-on way to test the command detection:

1. **Start the Demo**:
   ```bash
   cargo run
   ```

2. **Test Commands**: The prompt will show `>>` where you can enter text

3. **Smart Hints**: As you type, the system provides command completion hints

4. **Real-time Analysis**: Each input gets analyzed and classified immediately

### Example Session

```
Enter commands to check (type 'exit' to quit):
>> ls -la
Is shell command: true

>> what files are in this directory?
Is shell command: false

>> git status
Is shell command: true

>> can you help me find all text files?
Is shell command: false
```

## Relevant Examples

### Basic Command Detection

```rust
let mut detector = ShellCommandDetector::new();

// Shell commands return true
assert!(detector.is_shell_command("cd ~/projects"));
assert!(detector.is_shell_command("docker ps -a"));
assert!(detector.is_shell_command("./my_script.sh"));

// Natural language returns false  
assert!(!detector.is_shell_command("how do I list files?"));
assert!(!detector.is_shell_command("please show me the files"));
```

### Handling Complex Cases

The detector gracefully handles nuanced scenarios:

```rust
// Commands with quoted natural language
assert!(detector.is_shell_command("echo 'hello world'"));
assert!(detector.is_shell_command("grep 'search pattern' file.txt"));

// Questions and conversational input
assert!(!detector.is_shell_command("what is the current directory?"));
assert!(!detector.is_shell_command("can you help me with git?"));
```

### Getting Command Suggestions

```rust
let suggestions = detector.get_command_suggestions("gi");
// Returns: ["git", "gist", "gzip", ...] (commands starting with "gi")
```

## Architecture Insights

### Command Discovery Process

1. **PATH Scanning**: Discovers all executable files in system PATH directories
2. **Alias Loading**: Extracts user-defined aliases from the current shell
3. **Builtin Recognition**: Includes common shell builtins and popular commands
4. **Runtime Verification**: Uses `which` command for final validation

### Natural Language Heuristics

The system employs sophisticated pattern recognition:

- **Question Patterns**: Detects interrogative structures and question words
- **Conversational Markers**: Identifies polite language and helper phrases  
- **Comparative Language**: Recognizes descriptive and comparative expressions
- **Article Analysis**: Considers usage of articles with common word ratios

## Testing the Logic

```bash
cargo test
```


## Future Enhancements

This demo represents the core functionality with room for expansion:

- **Machine Learning Integration**: Train models on command/natural language datasets
- **Context Memory**: Remember user patterns and preferences
- **Plugin Architecture**: Support for domain-specific command recognition
