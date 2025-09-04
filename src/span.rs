use rhai::Position;

/// Represents a span in a Rhai script.
#[derive(Debug, Clone)]
pub struct Span {
    start: usize,
    end: usize,
    line: usize,
    column: usize,
}

impl Span {
    /// Create a new span from values
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Span {
            start,
            end,
            line,
            column,
        }
    }
    /// Get the start byte offset
    pub fn start(&self) -> usize {
        self.start
    }
    /// Get the end byte offset
    pub fn end(&self) -> usize {
        self.end
    }
    /// Get the line
    pub fn line(&self) -> usize {
        self.line
    }
    /// Get the column
    pub fn column(&self) -> usize {
        self.column
    }

    /// Create a Span from a Rhai `Position` and script text
    pub fn from_pos(script: &str, pos: &Position) -> Self {
        let line_idx = pos.line().expect("Position missing line") - 1;
        let column_idx = pos.position().expect("Position missing column") - 1;

        let start = script
            .lines()
            .take(line_idx)
            .map(|l| l.len() + 1)
            .sum::<usize>()
            + column_idx;

        let line_content = script.lines().nth(line_idx).unwrap_or("");
        let end = script
            .lines()
            .take(line_idx)
            .map(|l| l.len() + 1)
            .sum::<usize>()
            + line_content.len();

        Self {
            start,
            end,
            line: pos.line().expect("Position missing line"),
            column: pos.position().expect("Position missing column"),
        }
    }

    /// Create a span from rhai start end position.
    pub fn from_rhai_start_end_pos(script: &str, start: &Position, end: &Position) -> Self {
        let start_offset = pos_to_byte(script, start);
        let end_offset = pos_to_byte(script, end);

        Self {
            start: start_offset,
            end: end_offset,
            line: start.line().expect("Position missing line"),
            column: start.position().expect("Position missing column"),
        }
    }

    /// Convert Rhai's Span to our [`Span`]
    pub fn from_rhai_span(script: &str, rhai_span: rhai::Span, pos: &Position) -> Self {
        let start_byte = pos_to_byte(script, &rhai_span.start());
        let end_byte = pos_to_byte(script, &rhai_span.end());

        Self {
            start: start_byte,
            end: end_byte,
            line: pos.line().expect("Position missing line"),
            column: pos.position().expect("Position missing column"),
        }
    }
}

fn pos_to_byte(script: &str, pos: &Position) -> usize {
    let line_idx = pos.line().unwrap_or(1).saturating_sub(1);
    let col_idx = pos.position().unwrap_or(1).saturating_sub(1);

    script
        .lines()
        .take(line_idx)
        .map(|l| l.len() + 1)
        .sum::<usize>()
        + col_idx
}
