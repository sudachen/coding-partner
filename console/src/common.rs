use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Observability {
    pub statistics: bool,
    pub thinking: bool,
}

impl Default for Observability {
    fn default() -> Self {
        Self { statistics: false, thinking: false }
    }
}

#[derive(Error, Debug)]
pub enum ConsoleError {
    #[error("Input stream was terminated")]
    Terminated,
    #[error("Invalid state for this operation")]
    InvalidState,
    #[error("Unknown command: {command}")]
    UnknownCommand { command: String },
}

#[derive(Debug)]
pub enum ConsoleInput {
    Prompt {
        prompt: String,
    },
    Exit,
    Thinking(bool),
    Statistics(bool),
}

#[async_trait]
pub trait Console {
    async fn prompt_input(&mut self) -> Result<ConsoleInput, ConsoleError>;
    async fn start_responding(&mut self) -> Result<(), ConsoleError>;
    async fn stop_responding(&mut self) -> Result<(), ConsoleError>;
    fn observability(&mut self, on_off: Option<Observability>) -> Observability;
    async fn add_response_text(&mut self, text: String) -> Result<(), ConsoleError>;
    async fn add_thinking_text(&mut self, text: String) -> Result<(), ConsoleError>;
    async fn if_accept(&mut self, text: String) -> Result<bool, ConsoleError>;
    async fn if_yes(&mut self, text: String) -> Result<bool, ConsoleError>;
    async fn ask_user(&mut self, text: String) -> Result<String, ConsoleError>;
}
