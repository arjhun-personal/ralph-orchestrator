use tui_term::vt100::Parser;

pub struct TerminalWidget {
    parser: Parser,
}

impl Default for TerminalWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalWidget {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(24, 80, 0),
        }
    }

    pub fn process(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub fn parser(&self) -> &Parser {
        &self.parser
    }

    /// Returns total lines in scrollback.
    pub fn total_lines(&self) -> usize {
        let (rows, _cols) = self.parser.screen().size();
        self.parser.screen().scrollback() + rows as usize
    }

    /// Resizes the terminal.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.parser = Parser::new(rows, cols, 0);
    }

    /// Clears the terminal screen and scrollback.
    pub fn clear(&mut self) {
        let (rows, cols) = self.parser.screen().size();
        self.parser = Parser::new(rows, cols, 0);
    }
}
