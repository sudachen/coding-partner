use crate::common::{Console, ConsoleError, ConsoleInput, Observability};
use async_trait::async_trait;
use std::io::{BufRead, BufReader, Read, Write};
use std::time::Instant;

#[derive(Debug)]
struct RespondingState {
    start_time: Option<Instant>,
}

#[derive(Debug)]
enum State {
    Prompting,
    Responding(RespondingState),
}

pub struct StdioConsole<R, W>
where
    R: Read,
    W: Write,
{
    reader: BufReader<R>,
    writer: W,
    state: State,
    observability: Observability,
}

impl StdioConsole<std::io::Stdin, std::io::Stdout> {
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(std::io::stdin()),
            writer: std::io::stdout(),
            state: State::Prompting,
            observability: Observability::default(),
        }
    }
}

impl<R: Read, W: Write> StdioConsole<R, W> {
    pub fn new_with_buffers(reader: R, writer: W) -> Self {
        Self {
            reader: BufReader::new(reader),
            writer,
            state: State::Prompting,
            observability: Observability::default(),
        }
    }
}

#[async_trait]
impl<R, W> Console for StdioConsole<R, W>
where
    R: Read + Send,
    W: Write + Send,
{
    async fn prompt_input(&mut self) -> Result<ConsoleInput, ConsoleError> {
        if !matches!(self.state, State::Prompting) {
            return Err(ConsoleError::InvalidState);
        }
        let mut buffer = String::new();
        if self.reader.read_line(&mut buffer).is_ok() {
            let line = buffer.trim();
            if line.is_empty() {
                return Err(ConsoleError::Terminated);
            }

            if let Some(parts) = shlex::split(line) {
                if !parts.is_empty() && parts[0].starts_with('/') {
                    let command = parts[0].clone();
                    match parts.as_slice() {
                        [cmd, arg] if *cmd == "/thinking" => {
                            if *arg == "on" {
                                return Ok(ConsoleInput::Thinking(true));
                            } else if *arg == "off" {
                                return Ok(ConsoleInput::Thinking(false));
                            }
                        }
                        [cmd, arg] if *cmd == "/statistics" => {
                            if *arg == "on" {
                                return Ok(ConsoleInput::Statistics(true));
                            } else if *arg == "off" {
                                return Ok(ConsoleInput::Statistics(false));
                            }
                        }
                        [cmd] if *cmd == "/exit" => {
                            return Ok(ConsoleInput::Exit);
                        }
                        _ => {}
                    }
                    return Err(ConsoleError::UnknownCommand { command });
                }
            }

            Ok(ConsoleInput::Prompt { prompt: line.to_string() })
        } else {
            Err(ConsoleError::Terminated)
        }
    }

    async fn start_responding(&mut self) -> Result<(), ConsoleError> {
        if !matches!(self.state, State::Prompting) {
            return Err(ConsoleError::InvalidState);
        }
        self.state = State::Responding(RespondingState {
            start_time: if self.observability.statistics {
                Some(Instant::now())
            } else {
                None
            },
        });
        Ok(())
    }

    async fn stop_responding(&mut self) -> Result<(), ConsoleError> {
        let state = std::mem::replace(&mut self.state, State::Prompting);
        if let State::Responding(responding_state) = state {
            if self.observability.statistics {
                if let Some(start_time) = responding_state.start_time {
                    let duration = start_time.elapsed();
                    writeln!(self.writer, "Responding time: {:?}", duration)
                        .map_err(|_| ConsoleError::Terminated)?;
                    self.writer.flush().map_err(|_| ConsoleError::Terminated)?;
                }
            }
            Ok(())
        } else {
            self.state = state; // Restore state if it wasn't Responding
            Err(ConsoleError::InvalidState)
        }
    }

    fn observability(&mut self, on_off: Option<Observability>) -> Observability {
        let old = self.observability;
        if let Some(new_val) = on_off {
            self.observability = new_val;
        }
        old
    }

    async fn add_response_text(&mut self, text: String) -> Result<(), ConsoleError> {
        if !matches!(self.state, State::Responding(_)) {
            return Err(ConsoleError::InvalidState);
        }
        writeln!(self.writer, "{}", text).map_err(|_| ConsoleError::Terminated)?;
        self.writer.flush().map_err(|_| ConsoleError::Terminated)?;
        Ok(())
    }

    async fn add_thinking_text(&mut self, text: String) -> Result<(), ConsoleError> {
        if !matches!(self.state, State::Responding(_)) {
            return Err(ConsoleError::InvalidState);
        }
        if self.observability.thinking {
            writeln!(self.writer, "{}", text).map_err(|_| ConsoleError::Terminated)?;
            self.writer.flush().map_err(|_| ConsoleError::Terminated)?;
        }
        Ok(())
    }

    async fn if_accept(&mut self, text: String) -> Result<bool, ConsoleError> {
        if !matches!(self.state, State::Responding(_)) {
            return Err(ConsoleError::InvalidState);
        }
        self.if_yes(text).await
    }

    async fn if_yes(&mut self, text: String) -> Result<bool, ConsoleError> {
        if !matches!(self.state, State::Responding(_)) {
            return Err(ConsoleError::InvalidState);
        }
        loop {
            writeln!(self.writer, "{} [Y/n]", text).map_err(|_| ConsoleError::Terminated)?;
            self.writer.flush().map_err(|_| ConsoleError::Terminated)?;
            let mut buffer = String::new();
            if self.reader.read_line(&mut buffer).is_ok() {
                match buffer.trim().to_lowercase().as_str() {
                    "y" | "yes" | "" => return Ok(true),
                    "n" | "no" => return Ok(false),
                    _ => continue,
                }
            } else {
                return Err(ConsoleError::Terminated);
            }
        }
    }

    async fn ask_user(&mut self, text: String) -> Result<String, ConsoleError> {
        if !matches!(self.state, State::Responding(_)) {
            return Err(ConsoleError::InvalidState);
        }
        writeln!(self.writer, "{}", text).map_err(|_| ConsoleError::Terminated)?;
        self.writer.flush().map_err(|_| ConsoleError::Terminated)?;
        let mut buffer = String::new();
        if self.reader.read_line(&mut buffer).is_ok() {
            Ok(buffer.trim_end().to_string())
        } else {
            Err(ConsoleError::Terminated)
        }
    }
}