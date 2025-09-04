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

/// A `Console` implementation that uses standard I/O streams.
///
/// This struct provides a basic, text-based console interface by reading from a `Read`
/// buffer (like `stdin`) and writing to a `Write` buffer (like `stdout`). It's suitable
/// for command-line applications and testing purposes.
pub struct StdIo<R, W>
where
    R: Read,
    W: Write,
{
    reader: BufReader<R>,
    writer: W,
    state: State,
    observability: Observability,
}

impl StdIo<std::io::Stdin, std::io::Stdout> {
    /// Creates a new `StdIo` instance that reads from `stdin` and writes to `stdout`.
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(std::io::stdin()),
            writer: std::io::stdout(),
            state: State::Prompting,
            observability: Observability::default(),
        }
    }
}

impl<R: Read, W: Write> StdIo<R, W> {
    /// Creates a new `StdIo` instance with the given reader and writer buffers.
    ///
    /// This is useful for testing, allowing you to replace `stdin` and `stdout` with
    /// in-memory buffers.
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
/// Implements the `Console` trait for the `StdIo` struct.
///
/// This implementation provides a straightforward, synchronous console experience. It's
/// important to note that while the trait methods are `async`, this implementation
/// blocks on I/O operations.
impl<R, W> Console for StdIo<R, W>
where
    R: Read + Send,
    W: Write + Send,
{
    /// Prompts for user input by reading a line from the input buffer.
    ///
    /// The prompt itself is not displayed by this method; it's assumed to be handled
    /// by the calling context or the terminal's natural behavior.
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
            ConsoleInput::from_line(line)
        } else {
            Err(ConsoleError::Terminated)
        }
    }

    /// Switches the internal state to `Responding`.
    ///
    /// If statistics are enabled, it records the start time to calculate response
    /// duration later.
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

    /// Switches the internal state back to `Prompting`.
    ///
    /// If statistics are enabled, it calculates and prints the total response time.
    async fn stop_responding(&mut self) -> Result<(), ConsoleError> {
        let state = std::mem::replace(&mut self.state, State::Prompting);
        if let State::Responding(responding_state) = state {
            if self.observability.statistics {
                if let Some(start_time) = responding_state.start_time {
                    let duration = start_time.elapsed();
                    writeln!(self.writer, "Response time: {:?}", duration)
                        .map_err(|_| ConsoleError::Terminated)?;
                    writeln!(self.writer).map_err(|_| ConsoleError::Terminated)?;
                    self.writer.flush().map_err(|_| ConsoleError::Terminated)?;
                }
            }
            Ok(())
        } else {
            self.state = state; // Restore state if it wasn't Responding
            Err(ConsoleError::InvalidState)
        }
    }

    /// Gets or sets the observability flags.
    ///
    /// See `Console::observability` for detailed behavior.
    fn observability(&mut self, new_settings: Option<Observability>) -> Observability {
        let old = self.observability;
        if let Some(new_val) = new_settings {
            self.observability = new_val;
        }
        old
    }

    /// Appends a block of text to the agent's response area.
    ///
    /// This implementation writes the text followed by a newline to the output buffer.
    async fn add_response_text(&mut self, text: String) -> Result<(), ConsoleError> {
        if !matches!(self.state, State::Responding(_)) {
            return Err(ConsoleError::InvalidState);
        }
        writeln!(self.writer, "{}", text).map_err(|_| ConsoleError::Terminated)?;
        self.writer.flush().map_err(|_| ConsoleError::Terminated)?;
        Ok(())
    }

    /// Appends text to the agent's "thinking" status display if enabled.
    ///
    /// This writes the text to the output buffer only if the `thinking` flag in
    /// `Observability` is set to `true`.
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

    /// Asks the user for a confirmation, defaulting to "yes".
    ///
    /// It prints the given text with a `[Y/n]` prompt and waits for user input.
    /// 'y', 'yes', or an empty line are considered confirmations.
    async fn if_accept(&mut self, text: String) -> Result<bool, ConsoleError> {
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

    /// Asks the user a yes/no question, defaulting to "no".
    ///
    /// It prints the given text with a `[y/N]` prompt and waits for user input.
    /// Only 'y' or 'yes' are considered affirmative answers.
    async fn if_yes(&mut self, text: String) -> Result<bool, ConsoleError> {
        if !matches!(self.state, State::Responding(_)) {
            return Err(ConsoleError::InvalidState);
        }
        loop {
            writeln!(self.writer, "{} [y/N]", text).map_err(|_| ConsoleError::Terminated)?;
            self.writer.flush().map_err(|_| ConsoleError::Terminated)?;
            let mut buffer = String::new();
            if self.reader.read_line(&mut buffer).is_ok() {
                match buffer.trim().to_lowercase().as_str() {
                    "y" | "yes" => return Ok(true),
                    "n" | "no" | "" => return Ok(false),
                    _ => continue,
                }
            } else {
                return Err(ConsoleError::Terminated);
            }
        }
    }

    /// Prompts the user for a single line of text input.
    ///
    /// It prints the given prompt text and returns the user's trimmed input.
    async fn ask_user(&mut self, text: String) -> Result<String, ConsoleError> {
        self.writer.write_all(format!("{} ", text).as_bytes()).unwrap();
        self.writer.flush().unwrap();
        let mut buffer = String::new();
        if self.reader.read_line(&mut buffer).is_ok() {
            Ok(buffer.trim().to_string())
        } else {
            Err(ConsoleError::Terminated)
        }
    }
}