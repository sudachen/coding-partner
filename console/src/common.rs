use async_trait::async_trait;
pub use crate::input::ConsoleInput;

#[derive(Debug, thiserror::Error)]
pub enum ConsoleError {
    /// The console was terminated (Ctrl-C, Ctrl-D, redirected stream was closed ...)
    #[error("Console terminated")]
    Terminated,
    /// The console is in invalid state for the operation
    #[error("Invalid state for operation")]
    InvalidState,
    /// The command is unknown
    #[error("Unknown command: {command}")]
    UnknownCommand { command: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Observability {
    pub statistics: bool,
    pub thinking: bool,
}

/// A trait for interacting with the user console.
///
/// This trait defines a standard interface for console operations, allowing for different
/// implementations, such as a terminal-based console or a graphical user interface.
/// It handles user input, agent output, and various interactive prompts.
#[async_trait]
pub trait Console {
    /// Prompts the user for input and returns the parsed command or text.
    ///
    /// This function will wait for the user to submit a line of input. The input is then
    /// parsed into a `ConsoleInput` enum, which can be a command, text, or termination
    /// signal.
    async fn prompt_input(&mut self) -> Result<ConsoleInput, ConsoleError>;

    /// Switches the console to "responding" mode.
    ///
    /// In this mode, the console typically displays a streaming response from the agent and
    /// may hide the user input prompt.
    async fn start_responding(&mut self) -> Result<(), ConsoleError>;

    /// Switches the console back to "input" mode.
    ///
    /// This function is called after the agent has finished sending its response. The user
    /// input prompt is typically displayed again.
    async fn stop_responding(&mut self) -> Result<(), ConsoleError>;

    /// Gets or sets the observability flags for debugging and statistics.
    ///
    /// If `new_settings` is `Some`, it sets the observability flags to the given values
    /// and returns the *previous* settings. If `new_settings` is `None`, it returns the
    /// current settings without changing them.
    fn observability(&mut self, new_settings: Option<Observability>) -> Observability;

    /// Appends a block of text to the agent's response area.
    ///
    /// This is the primary way for the agent to communicate its results to the user.
    async fn add_response_text(&mut self, text: String) -> Result<(), ConsoleError>;

    /// Appends text to the agent's "thinking" status display.
    ///
    /// This can be used to show progress or internal status to the user while the agent
    /// is working on a response.
    async fn add_thinking_text(&mut self, text: String) -> Result<(), ConsoleError>;

    /// Asks the user for a confirmation on a specific action.
    ///
    /// The `text` parameter contains the question or action to be confirmed. Returns `true`
    /// if the user provides a positive confirmation (e.g., 'y' or 'yes').
    async fn if_accept(&mut self, text: String) -> Result<bool, ConsoleError>;

    /// Asks the user a yes/no question.
    ///
    /// The `text` parameter contains the question. Returns `true` if the user answers
    /// 'yes'. This is similar to `if_accept` but may have a different presentation.
    async fn if_yes(&mut self, text: String) -> Result<bool, ConsoleError>;

    /// Prompts the user for a single line of text input.
    ///
    /// The `text` parameter is the prompt displayed to the user. Returns the string
    /// entered by the user.
    async fn ask_user(&mut self, text: String) -> Result<String, ConsoleError>;
}
