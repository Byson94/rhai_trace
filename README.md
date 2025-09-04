# rhai_trace

`rhai_trace` is a lightweight Rust library that enhances [Rhai](https://rhai.rs) scripts with **better error reporting and span tracking**. It walks the script's AST, extracts spans for statements and expressions, and provides structured error information that can be used with any diagnostic or pretty-printing crate.

With `rhai_trace`, you can:

- Extract spans (start/end byte offsets, line, column) from Rhai scripts.
- Get detailed runtime error diagnostics including messages, hints, and notes.
- Integrate easily with crates like [`ariadne`](https://docs.rs/ariadne) or other diagnostic systems.

## Quick Example

```rust
use rhai_trace::{SpanTracer, BetterError};
use rhai::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example Rhai code
    let code = r#"
        let a = 42;
        let b = a + 1;
        fn multiply(x, y) { x * y }
        let c = multiply("a", 7);  // <-- will trigger runtime error
    "#;

    // Initialize the span tracer
    let tracer = SpanTracer::new();

    // Extract spans from the code
    let spans = tracer.extract_from(code)?;

    println!("Extracted spans:");
    for span in &spans {
        println!("{}..{}: '{}'",
            span.start(),
            span.end(),
            &code[span.start()..span.end()]
        );
    }

    // Attempt to execute the code with Rhai engine
    let engine = Engine::new();
    match engine.eval::<rhai::Dynamic>(code) {
        Ok(result) => println!("Execution result: {:?}", result),
        Err(e) => {
            // Improve the error using our library
            if let Ok(better) = BetterError::improve_eval_error(&e, code, &engine) {
                // ...
            } else {
                eprintln!("Original Error: {:?}", e);
            }
        }
    }

    Ok(())
}
```

## Full Example

For a complete working example showing integration with [`ariadne`](https://docs.rs/ariadne) for pretty error reporting, see the `example` folder in the repository:

[GitHub Example](https://github.com/Byson94/rhai_trace/tree/main/example)
