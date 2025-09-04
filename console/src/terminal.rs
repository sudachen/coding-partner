use crate::common::{Console, ConsoleError, ConsoleInput, Observability};
use async_trait::async_trait;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal as RatatuiTerminal,
};
use std::io::{self, Stdout};

enum Mode {
    Prompting,
    Responding,
}

struct State {
    mode: Mode,
    observability: Observability,
    input_text: String,
    messages: Vec<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: Mode::Prompting,
            observability: Observability::default(),
            input_text: String::new(),
            messages: Vec::new(),
        }
    }
}

pub struct Terminal {
    terminal: RatatuiTerminal<CrosstermBackend<Stdout>>,
    state: State,
}

impl Terminal {
    pub fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        io::stdout().execute(EnterAlternateScreen)?;
        let terminal = RatatuiTerminal::new(CrosstermBackend::new(io::stdout()))?;
        Ok(Self {
            terminal,
            state: State::default(),
        })
    }

    fn draw(&mut self) -> Result<(), io::Error> {
        self.terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                .split(frame.size());

            let messages = self.state.messages.join("\n");
            let message_paragraph = Paragraph::new(messages)
                .block(Block::default().borders(Borders::ALL).title("Messages"));
            frame.render_widget(message_paragraph, chunks[0]);

            let input_paragraph = Paragraph::new(self.state.input_text.as_str())
                .block(Block::default().borders(Borders::ALL).title("Input"));
            frame.render_widget(input_paragraph, chunks[1]);
        })?;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // It's good practice to ignore errors during drop, as panicking in drop is problematic.
        let _ = io::stdout().execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

#[async_trait]
impl Console for Terminal {
    async fn prompt_input(&mut self) -> Result<ConsoleInput, ConsoleError> {
        loop {
            self.draw().map_err(|_| ConsoleError::Terminated)?;

            if event::poll(std::time::Duration::from_millis(100))
                .map_err(|_| ConsoleError::Terminated)?
            {
                if let Event::Key(key) = event::read().map_err(|_| ConsoleError::Terminated)? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Enter => {
                                let line = self.state.input_text.clone();
                                self.state.input_text.clear();
                                self.state.messages.push(format!("> {}", line));
                                return ConsoleInput::from_line(&line);
                            }
                            KeyCode::Char(c) => {
                                self.state.input_text.push(c);
                            }
                            KeyCode::Backspace => {
                                self.state.input_text.pop();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    async fn start_responding(&mut self) -> Result<(), ConsoleError> {
        self.state.mode = Mode::Responding;
        Ok(())
    }

    async fn stop_responding(&mut self) -> Result<(), ConsoleError> {
        self.state.mode = Mode::Prompting;
        Ok(())
    }

    fn observability(&mut self, on_off: Option<Observability>) -> Observability {
        if let Some(new_observability) = on_off {
            self.state.observability = new_observability;
        }
        self.state.observability
    }

    async fn add_response_text(&mut self, text: String) -> Result<(), ConsoleError> {
        self.state.messages.push(text);
        self.draw().map_err(|_| ConsoleError::Terminated)
    }

    async fn add_thinking_text(&mut self, text: String) -> Result<(), ConsoleError> {
        // For now, just add it to messages. We can make this fancier later.
        if self.state.observability.thinking {
            self.state.messages.push(format!("[thinking] {}", text));
            self.draw().map_err(|_| ConsoleError::Terminated)?;
        }
        Ok(())
    }

    async fn if_accept(&mut self, text: String) -> Result<bool, ConsoleError> {
        self.if_yes(text).await
    }

    async fn if_yes(&mut self, text: String) -> Result<bool, ConsoleError> {
        // This is a simplified implementation for the terminal UI.
        // A real implementation would involve a modal dialog or similar.
        self.state.messages.push(format!("{} [Y/n]", text));
        self.draw().map_err(|_| ConsoleError::Terminated)?;

        loop {
            if event::poll(std::time::Duration::from_millis(100))
                .map_err(|_| ConsoleError::Terminated)?
            {
                if let Event::Key(key) = event::read().map_err(|_| ConsoleError::Terminated)? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                                return Ok(true);
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                return Ok(false);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    async fn ask_user(&mut self, _text: String) -> Result<String, ConsoleError> {
        unimplemented!()
    }
}