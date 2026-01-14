//! Main application loop for the TUI.

use crate::input::{Command, InputRouter, RouteResult};
use crate::state::TuiState;
use crate::widgets::{footer, header, help, terminal::TerminalWidget};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ralph_adapters::pty_handle::PtyHandle;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use std::io;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, interval};

/// Main TUI application.
pub struct App {
    state: Arc<Mutex<TuiState>>,
    terminal_widget: Arc<Mutex<TerminalWidget>>,
    input_router: InputRouter,
}

impl App {
    /// Creates a new App with shared state and PTY handle.
    pub fn new(state: Arc<Mutex<TuiState>>, pty_handle: PtyHandle) -> Self {
        let terminal_widget = Arc::new(Mutex::new(TerminalWidget::new()));

        // Spawn task to read PTY output and feed to terminal widget
        let widget_clone = Arc::clone(&terminal_widget);
        tokio::spawn(async move {
            let PtyHandle { mut output_rx, .. } = pty_handle;
            while let Some(bytes) = output_rx.recv().await {
                if let Ok(mut widget) = widget_clone.lock() {
                    widget.process(&bytes);
                }
            }
        });

        Self {
            state,
            terminal_widget,
            input_router: InputRouter::new(),
        }
    }

    /// Runs the TUI event loop.
    pub async fn run(mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut tick = interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                _ = tick.tick() => {
                    let state = self.state.lock().unwrap();
                    let widget = self.terminal_widget.lock().unwrap();
                    terminal.draw(|f| {
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Length(3),
                                Constraint::Min(0),
                                Constraint::Length(3),
                            ])
                            .split(f.area());

                        f.render_widget(header::render(&state), chunks[0]);
                        f.render_widget(tui_term::widget::PseudoTerminal::new(widget.parser().screen()), chunks[1]);
                        f.render_widget(footer::render(&state), chunks[2]);

                        if state.show_help {
                            help::render(f, f.area());
                        }
                    })?;

                    // Poll for keyboard events
                    if event::poll(Duration::from_millis(0))? {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press {
                                // Dismiss help on any key
                                if self.state.lock().unwrap().show_help {
                                    self.state.lock().unwrap().show_help = false;
                                    continue;
                                }

                                match self.input_router.route_key(key) {
                                    RouteResult::Forward(_) => {
                                        // TODO: Forward to PTY in next step
                                    }
                                    RouteResult::Command(cmd) => {
                                        match cmd {
                                            Command::Quit => break,
                                            Command::Help => {
                                                self.state.lock().unwrap().show_help = true;
                                            }
                                            Command::Unknown => {}
                                        }
                                    }
                                    RouteResult::Consumed => {
                                        // Prefix consumed, wait for command
                                    }
                                }
                            }
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        Ok(())
    }
}
