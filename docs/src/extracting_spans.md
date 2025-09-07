# Extracting Spans

The `SpanTracer` is a powerful utility in `rhai_trace` that **extracts spans from Rhai scripts**, mapping every statement or expression to its exact location in the source code. It provides byte offsets, line numbers, and column positions, enabling precise diagnostics, navigation, and analysis.

# Purpose

- Parses scripts to identify code segments
- Associates syntax elements with their location in the source text
- Facilitates error highlighting, debugging tools, and developer feedback mechanisms

## Example Usage

```rust, ignore
use rhai_trace::{SpanTracer, Span};

let code = r#"
    let a = 1;
    let b = a + 2;
"#;

let tracer = SpanTracer::new();
let spans = tracer.extract_from(code).unwrap();

for span in spans {
    println!("Span: {}..{} (line {}, column {})",
             span.start(), span.end(), span.line(), span.column());
}
```

This outputs each segment’s location, allowing tools to provide targeted information based on where the error or expression occurs.

## Methods

- `SpanTracer::new()`
  Creates a new `SpanTracer` instance.

- `SpanTracer::extract_from(script)`
  Analyzes the provided script and returns a `Vec<Span>` containing all identified spans. Fails with an error if the script cannot be parsed.
