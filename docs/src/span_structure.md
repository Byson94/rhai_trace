# `Span` Structure

The `Span` structure is a **core component of `rhai_trace`**, representing a contiguous segment of source code with both byte-level precision and human-readable positioning. It’s essential for error reporting, diagnostics, and integration with external tools like **ariadne** for visual feedback.

## Key Features

- **Byte Offsets**: `start` and `end` mark the exact location in the source text.
- **Line & Column**: Provides 1-based human-readable context for developers.
- **Use Cases**: Highlights errors, annotates code, and feeds structured information into debugging or visualization tools.

## Example Usage

```rust, ignore
use rhai_trace::Span;

let span = Span::new(10, 20, 2, 5);
println!(
    "Span covers bytes {}..{} on line {}",
    span.start(),
    span.end(),
    span.line()
);
```

## Constructors & Methods

- `Span::new(start, end, line, column)`
  Creates a span from byte offsets, line, and column information.

- `span.start()` / `span.end()`
  Returns the starting and ending byte offsets.

- `span.line()` / `span.column()`
  Returns the line and column numbers (1-based).

- `Span::from_pos(script, pos)`
  Converts a rhai `Position` and source text into a `Span` by calculating byte offsets.

- `Span::from_rhai_start_end_pos(script, start, end)`
  Creates a span from two Rhai `Position`.

- `Span::from_rhai_span(script, rhai_span, pos)`
  Converts a Rhai `Span` into `rhai_trace`’s `Span` using contextual information.

## Why It is important

The `Span` structure bridges machine-level parsing and human-centric debugging.
