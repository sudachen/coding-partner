use crate::common::ConsoleError;

#[derive(Debug, PartialEq, Eq)]
pub enum ConsoleInput {
    Prompt { prompt: String },
    Exit,
    Thinking(bool),
    Statistics(bool),
}

impl ConsoleInput {
    pub fn from_line(line: &str) -> Result<Self, ConsoleError> {
        let line = line.trim();
        if let Some(parts) = shlex::split(line) {
            if !parts.is_empty() && parts[0].starts_with('/') {
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
        Ok(ConsoleInput::Prompt {
            prompt: line.to_string(),
        })
    }
}
