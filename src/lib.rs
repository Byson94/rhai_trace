//! `rhai_trace` is a small Rust library which provides better error and span support 
//! for [Rhai](https://rhai.rs), the embeddable scripting language.
//!
//! # Example
//!
//! ```rust
//! use rhai_trace::{SpanTracer, BetterError};
//! use rhai::Engine;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Example Rhai code
//!     let code = r#"
//!         let a = 42;
//!         let b = a + 1;
//!         fn multiply(x, y) { x * y }
//!         let c = multiply("a", 7);  // <-- will trigger runtime error
//!     "#;
//!
//!     // Initialize the span tracer
//!     let tracer = SpanTracer::new();
//!
//!     // Extract spans from the code
//!     let spans = tracer.extract_from(code)?;
//!
//!     println!("Extracted spans:");
//!     for span in &spans {
//!         for span in &spans {
//!             println!("{}..{}: '{}'",
//!                 span.start(),
//!                 span.end(),
//!                 &code[span.start()..span.end()]
//!             );
//!         }
//!     }
//!
//!     // Attempt to execute the code with Rhai engine
//!     let engine = Engine::new();
//!     match engine.eval::<rhai::Dynamic>(code) {
//!         Ok(result) => println!("Execution result: {:?}", result),
//!         Err(e) => {
//!             // Improve the error using our library
//!             if let Ok(better) = BetterError::improve_eval_error(&e, code, &engine) {
//!                 // This returns a [`BetterError`] structure.
//!                 //
//!                 // It includes all the information you need to print a
//!                 // pretty error! It provides all the information needed to display
//!                 // diagnostics (message, hint, note, etc.) and can be used with any
//!                 // diagnostic or pretty-printing crate, even those that 
//!                 // don't natively support spans.
//!                 //
//!                 // A full example showing this integration is provided below. 
//                  // See the repository link at the end of this page for a full working example.
//!             } else {
//!                 eprintln!("Original Error: {:?}", e);
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//!
//! For a complete working example that integrates `rhai_trace` with the [`ariadne`](https://docs.rs/ariadne) crate for pretty error reporting, check out the example folder:
//! [GitHub Example](https://github.com/Byson94/rhai_trace/tree/main/example)

pub mod error;
pub mod span;
pub mod tracer;

// == Rexporting ==//
pub use error::BetterError;
pub use span::Span;
pub use tracer::SpanTracer;

#[cfg(test)]
mod test {
    use super::*;
    use rhai::{Engine, Dynamic};

    #[test]
    fn test_span_extraction() {
        let code = r#"
            let a = 42;
            let b = a + 1;
            fn add(x, y) { x + y }
            let z = a/0;
            let c = add(a, b);
        "#;

        let tracer = SpanTracer::new();
        let spans_result = tracer.extract_from(code);

        match spans_result {
            Ok(_) => {}
            Err(ref err) => {
                if let Some(parse_err) = err.downcast_ref::<rhai::ParseError>() {
                    match BetterError::improve_parse_error(&parse_err, code) {
                        Ok(better_error) => println!("Better error: {:?}", better_error),
                        Err(e) => eprintln!("Failed to improve parse error: {:?}", e),
                    }

                    return;
                } else {
                    eprintln!("Other error: {:?}", err);
                    return;
                }
            }
        }

        let spans = spans_result.unwrap();
        assert!(!spans.is_empty(), "There should be some spans extracted");

        //  Check that all spans have valid start/end
        for span in &spans {
            assert!(span.start() <= span.end(), "Span start should be <= end");
            assert!(
                span.end() <= code.len(),
                "Span end should not exceed code length"
            );

            // Check that span content is not empty
            let snippet = &code[span.start()..span.end()];
            assert!(
                !snippet.trim().is_empty(),
                "Span content should not be empty"
            );
        }

        // Debug output
        for span in &spans {
            println!("Span: {:?} -> '{}'", span, &code[span.start()..span.end()]);
        }
    }

    #[test]
    fn test_better_error() {
        let code = r#"
fn multiply(x, y) { x * y }

multiply("a", 2);

return "test complete"
        "#;

        let engine = Engine::new();
        let mut scope = rhai::Scope::new();
        engine.eval_with_scope::<Dynamic>(&mut scope, code).map_err(|e| {
            eprintln!(
                "Better Error: {:#?}",
                BetterError::improve_eval_error(&e, code, &engine, None)
            );
        });
    }
}
