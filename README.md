# rhai_trace

`rhai-trace` is a simple rust library for providing better error support for [Rhai](https://rhai.rs), the embeddable programming language.

# Example

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
        println!("{}..{}: '{}'", span.start(), span.end(), &code[span.start()..span.end()]);
    }
    // Attempt to execute the code with Rhai engine
    let engine = Engine::new();
    match engine.eval::<rhai::Dynamic>(code) {
        Ok(result) => println!("Execution result: {:?}", result),
        Err(e) => {
            // Improve the error using our library
            if let Ok(better) = BetterError::improve_eval_error(&e, code, &engine) {
                // This returns a [`BetterError`] structure.
                //
                // It includes all the information you need to print a
                // pretty error! It provides all the information needed to display
                // diagnostics (message, hint, note, etc.) and can be used with any
                // diagnostic or pretty-printing crate, even those that
                // don't natively support spans.
                //
                // A full example showing this integration is provided below.
                // See the repository link at the end of this page for a full working example.
            } else {
                eprintln!("Original Error: {:?}", e);
            }
        }
    }
    Ok(())
}
```

For a complete working example that integrates `rhai_trace` with the [`ariadne`](https://docs.rs/ariadne) crate for pretty error reporting, check out the example folder:
[GitHub Example](https://github.com/Byson94/rhai_trace/tree/main/example)
