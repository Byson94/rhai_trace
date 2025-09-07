# Propagating Better Errors

`rhai_trace` enables **structured, insightful error reporting** by transforming raw Rhai errors into enriched diagnostics. Instead of handling opaque error messages, developers can seamlessly integrate contextualized error reports into tooling like IDEs or debuggers.

## Core Concept: `BetterError`

At its heart is the `BetterError` struct, designed to aggregate error metadata:

```rust, ignore
pub struct BetterError {
    pub message: String,
    pub help: Option<String>,
    pub hint: Option<String>,
    pub note: Option<String>,
    pub span: Span,
}
```

- **Message**: what went wrong (original error)
- **Help**: actionable suggestions
- **Hint**: contextual nudges
- **Note**: additional insights
- **Span**: location in source code

`BetterError` makes it possible to enhance diagnostics with code context or execution.

## Improving Errors

`rhai_trace` offers two functions for enhancing diagnostics:

### `BetterError::improve_eval_error`

Used for runtime errors where the script was successfully parsed and spans can be extracted.

```rust, ignore
if let Ok(better) = BetterError::improve_eval_error(
    &e,
    &code,
    &engine,
    None // <-- can replace with spans from `SpanTracker`
) {
    // Use `better` to enrich error reporting in tooling
} else {
    eprintln!("Original Error: {:?}", e);
}
```

If you want `improve_eval_error` to not extract spans every time, then you can pass the result from `SpanTracer` as the fourth argument.

This may be useful if you want to improve performance by caching the spans and reusing it when needed.

### `BetterError::improve_parse_error`

Used for syntax errors where the script failed to compile and spans cannot be extracted.

```rust, ignore
let ast = engine.compile(code)
    .map_err(|e| BetterError::improve_parse_error(&e, &code, &engine));
```

`improve_parse_error` does **not** require spans because parsing failed before code locations could be reliably extracted.

## Practical Example

```rust, ignore
use rhai_trace::*;

fn main() {
    let code = r#"
        fn multiply(x, y) { x * y }
        print(multiply("a", 7)); // triggers runtime error
    "#;

    let engine = Engine::new();

    match engine.eval::<rhai::Dynamic>(code) {
        Ok(_) => println!("Executed successfully"),
        Err(e) => {
            if let Ok(better) = BetterError::improve_eval_error(
                &e,
                code,
                &engine,
                None
            ) {
                // `better` == `BetterError`
                //
                // Each value can be extracted and
                // piped to diagnostic tools.
                plug_into_codespan(better);
            } else {
                eprintln!("Original Error: {:?}", e);
            }
        }
    }
}
```
