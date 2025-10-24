use rhai::Position;

/// Represents a contiguous segment of source code.
///
/// `start` and `end` are **byte offsets** into the source code,
/// while `line` and `column` represent the human-readable
/// position of the span.
///
/// This structure can be used for highlighting errors, diagnostics,
/// or feeding external crates like [`ariadne`](https://docs.rs/ariadne/latest/ariadne/).
///
/// # Example
///
/// ```rust
/// use rhai_trace::Span;
///
/// let span = Span::new(10, 20, 2, 5);
/// println!("Span covers bytes {}..{} on line {}", span.start(), span.end(), span.line());
/// ```
#[derive(Debug, Clone)]
pub struct Span {
    start: usize,
    end: usize,
    line: usize,
    column: usize,
}

impl Span {
    /// Creates a new `Span` from byte offsets, line, and column.
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Span {
            start,
            end,
            line,
            column,
        }
    }
    /// Returns the starting byte offset of this span.
    pub fn start(&self) -> usize {
        self.start
    }
    /// Returns the starting byte offset of this span.
    pub fn end(&self) -> usize {
        self.end
    }
    /// Returns the line number (1-based) of this span.
    pub fn line(&self) -> usize {
        self.line
    }
    /// Returns the column number (1-based) of this span.
    pub fn column(&self) -> usize {
        self.column
    }

    /// Creates a `Span` from a Rhai `Position` and the script text.
    /// Computes byte offsets based on line and column.
    pub fn from_pos(script: &str, pos: &Position) -> Self {
        if pos.is_none() {
            return Self {
                start: 0,
                end: 0,
                line: 0,
                column: 0,
            };
        }
        
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

    /// Creates a `Span` from Rhai start and end `Position`s.
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

    /// Converts a Rhai `Span` to our `Span` type using a reference `Position`.
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
