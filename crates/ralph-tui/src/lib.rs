//! # ralph-tui
//!
//! Terminal user interface for the Ralph Orchestrator framework.
//!
//! Built with `ratatui` and `crossterm`, this crate provides:
//! - Interactive terminal UI for monitoring agent orchestration
//! - Real-time display of agent messages and state
//! - Keyboard navigation and input handling

mod app;
pub mod input;
pub mod scroll;
pub mod state;
pub mod widgets;

use anyhow::Result;
use app::App;
use ralph_adapters::pty_handle::PtyHandle;
use ralph_proto::Event;
use state::TuiState;
use std::sync::{Arc, Mutex};

pub use widgets::terminal::TerminalWidget;

/// Main TUI handle that integrates with the event bus.
pub struct Tui {
    state: Arc<Mutex<TuiState>>,
    pty_handle: Option<PtyHandle>,
}

impl Tui {
    /// Creates a new TUI instance with shared state.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(TuiState::new())),
            pty_handle: None,
        }
    }

    /// Sets the PTY handle for terminal output.
    pub fn with_pty(mut self, pty_handle: PtyHandle) -> Self {
        self.pty_handle = Some(pty_handle);
        self
    }

    /// Returns an observer closure that updates TUI state from events.
    pub fn observer(&self) -> impl Fn(&Event) + Send + 'static {
        let state = Arc::clone(&self.state);
        move |event: &Event| {
            if let Ok(mut s) = state.lock() {
                s.update(event);
            }
        }
    }

    /// Runs the TUI application loop.
    pub async fn run(self) -> Result<()> {
        let pty_handle = self
            .pty_handle
            .expect("PTY handle not set - call with_pty() first");
        let app = App::new(Arc::clone(&self.state), pty_handle);
        app.run().await
    }
}

impl Default for Tui {
    fn default() -> Self {
        Self::new()
    }
}
