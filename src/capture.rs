use chrono::{DateTime, Local};
use std::path::PathBuf;

/// A captured command with its execution context and output
#[derive(Debug, Clone)]
pub struct CapturedCommand {
    /// The command that was executed
    pub command: String,

    /// Full output (stdout + stderr combined)
    pub output: String,

    /// Exit code of the command
    pub exit_code: Option<i32>,

    /// Timestamp when command was entered
    pub timestamp: DateTime<Local>,

    /// Working directory when command was executed
    pub working_dir: PathBuf,
}

impl CapturedCommand {
    pub fn new(command: String, working_dir: PathBuf) -> Self {
        Self {
            command,
            output: String::new(),
            exit_code: None,
            timestamp: Local::now(),
            working_dir,
        }
    }

    /// Add output chunk to this command's output
    pub fn append_output(&mut self, data: &str) {
        self.output.push_str(data);
    }

    /// Set the exit code when command completes
    pub fn set_exit_code(&mut self, code: i32) {
        self.exit_code = Some(code);
    }

    /// Check if this command is complete (has exit code)
    pub fn is_complete(&self) -> bool {
        self.exit_code.is_some()
    }
}

/// Manages the capture and storage of command executions
pub struct CommandCapture {
    /// List of all captured commands in this session
    commands: Vec<CapturedCommand>,

    /// Current command being captured (if any)
    current_command: Option<CapturedCommand>,

    /// Buffer for detecting prompts and commands in output
    output_buffer: String,
}

impl CommandCapture {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_command: None,
            output_buffer: String::new(),
        }
    }

    /// Process a chunk of output from the PTY
    /// Detects OSC 133 sequences for command tracking
    pub fn process_output(&mut self, data: &str, working_dir: &std::path::Path) -> bool {
        self.output_buffer.push_str(data);

        // Check for OSC 133 sequences first (most reliable)
        self.parse_osc_sequences(data, working_dir);

        // Keep buffer manageable (last 4KB should be enough for prompt detection)
        if self.output_buffer.len() > 4096 {
            self.output_buffer.drain(..self.output_buffer.len() - 4096);
        }

        // If we have a current command, append output to it (filter out OSC sequences)
        let clean_output = self.strip_osc_sequences(data);
        if let Some(ref mut cmd) = self.current_command {
            cmd.append_output(&clean_output);
        }

        // Fallback: Check if this looks like a new prompt
        self.detect_prompt()
    }

    /// Parse OSC 133 sequences for shell integration
    fn parse_osc_sequences(&mut self, data: &str, working_dir: &std::path::Path) {
        // OSC 133;C;command - Command about to execute
        if let Some(start) = data.find("\x1b]133;C;") {
            if let Some(end) = data[start..].find('\x07') {
                let command = &data[start + 8..start + end];
                eprintln!("\n[HOOK] preexec: {:?}", command);

                // Start new command capture
                if let Some(cmd) = self.current_command.take() {
                    eprintln!("[CAPTURE] ✓ {:?} → {} bytes captured", cmd.command, cmd.output.len());
                    self.commands.push(cmd);
                }
                self.current_command = Some(CapturedCommand::new(command.to_string(), working_dir.to_path_buf()));
            }
        }

        // OSC 133;D;exitcode - Command finished
        if let Some(start) = data.find("\x1b]133;D;") {
            if let Some(end) = data[start..].find('\x07') {
                let exit_code_str = &data[start + 8..start + end];
                if let Ok(exit_code) = exit_code_str.parse::<i32>() {
                    eprintln!("[HOOK] precmd: exit_code={}", exit_code);
                    if let Some(ref mut cmd) = self.current_command {
                        cmd.set_exit_code(exit_code);
                    }
                }
            }
        }
    }

    /// Strip OSC sequences from output to avoid polluting captured data
    fn strip_osc_sequences(&self, data: &str) -> String {
        let mut result = data.to_string();

        // Remove OSC 133 sequences
        while let Some(start) = result.find("\x1b]133;") {
            if let Some(end) = result[start..].find('\x07') {
                result.drain(start..start + end + 1);
            } else {
                break;
            }
        }

        result
    }

    /// Detect if the current buffer ends with a shell prompt
    /// Handles various prompt styles including oh-my-zsh
    fn detect_prompt(&self) -> bool {
        let trimmed = self.output_buffer.trim_end();

        if let Some(last_line) = trimmed.lines().last() {
            // Check for various prompt indicators

            // 1. Standard prompts ending with % or $
            if last_line.ends_with("% ") || last_line.ends_with("$ ") {
                return true;
            }

            // 2. Oh-my-zsh style prompts with arrows and git info
            // Pattern: "➜  directory git:(branch) ✗"
            if last_line.contains("➜") && last_line.len() < 200 {
                return true;
            }

            // 3. Prompts ending with special characters (λ, ❯, >, etc.)
            let prompt_endings = ["λ ", "❯ ", "> ", "→ ", "» ", "✗ "];
            for ending in &prompt_endings {
                if last_line.ends_with(ending) {
                    return true;
                }
            }

            // 4. ANSI escape sequences indicating cursor at start of line
            // This happens when the shell resets cursor position
            if trimmed.contains("\x1b[0m") && last_line.len() < 200 {
                // Reset sequence followed by short line often indicates prompt
                return true;
            }
        }

        false
    }

    /// Start capturing a new command
    pub fn start_command(&mut self, command: String, working_dir: PathBuf) {
        // If there was a previous command, finalize it
        if let Some(cmd) = self.current_command.take() {
            eprintln!("[CAPTURE] ✓ {:?} → {} bytes captured", cmd.command, cmd.output.len());
            self.commands.push(cmd);
        }

        // Start new command capture
        self.current_command = Some(CapturedCommand::new(command, working_dir));
    }

    /// Finalize the current command with an exit code
    pub fn finalize_command(&mut self, exit_code: i32) {
        if let Some(ref mut cmd) = self.current_command {
            cmd.set_exit_code(exit_code);
        }
    }

    /// Get all captured commands
    pub fn get_commands(&self) -> &[CapturedCommand] {
        &self.commands
    }

    /// Get the current command being captured
    pub fn current(&self) -> Option<&CapturedCommand> {
        self.current_command.as_ref()
    }

    /// Clear all captured commands (for testing or reset)
    pub fn clear(&mut self) {
        self.commands.clear();
        self.current_command = None;
        self.output_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_detection() {
        let mut capture = CommandCapture::new();

        // Test zsh prompt
        assert!(!capture.process_output("some output\n"));
        assert!(capture.process_output("user@host:~/projects % "));

        // Test simple prompt
        capture.clear();
        assert!(capture.process_output("~ % "));
    }

    #[test]
    fn test_command_capture() {
        let mut capture = CommandCapture::new();
        let cwd = PathBuf::from("/home/user");

        capture.start_command("ls -la".to_string(), cwd.clone());
        capture.process_output("total 32\ndrwxr-xr-x  5 user\n");
        capture.finalize_command(0);

        assert_eq!(capture.current().unwrap().exit_code, Some(0));
        assert!(capture.current().unwrap().output.contains("total 32"));
    }
}
