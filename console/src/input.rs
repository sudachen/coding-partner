use crate::common::ConsoleError;

#[derive(Debug, PartialEq, Eq)]
/// the user input can be a command or a prompt text
pub enum ConsoleInput {
    /// the user prompt
    Prompt { prompt: String },
    /// the command /exit
    Exit,
    /// the command /thinking on/off
    Thinking(bool),
    /// the command /statistics on/off
    Statistics(bool),
    /// any unknown command starting with '/' symbol
    UnknownCommand { command: String },
}

impl ConsoleInput {
    pub fn from_line(line: &str) -> Result<Self, ConsoleError> {
        let trimmed_line = line.trim();
        if trimmed_line.starts_with('/') {
            if let Some(parts) = shlex::split(trimmed_line) {
                if !parts.is_empty() {
                    let command = parts[0].clone();
                    return match parts.as_slice() {
                        [cmd, arg] if cmd.as_str() == "/thinking" => match arg.as_str() {
                            "on" => Ok(ConsoleInput::Thinking(true)),
                            "off" => Ok(ConsoleInput::Thinking(false)),
                            _ => Err(ConsoleError::UnknownCommand { command }),
                        },
                        [cmd, arg] if cmd.as_str() == "/statistics" => match arg.as_str() {
                            "on" => Ok(ConsoleInput::Statistics(true)),
                            "off" => Ok(ConsoleInput::Statistics(false)),
                            _ => Err(ConsoleError::UnknownCommand { command }),
                        },
                        [cmd] if cmd.as_str() == "/exit" => Ok(ConsoleInput::Exit),
                        _ => Err(ConsoleError::UnknownCommand { command }),
                    };
                }
            }
        }

        Ok(ConsoleInput::Prompt {
            prompt: line.to_string(),
        })
    }
}
