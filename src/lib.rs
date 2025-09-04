//! `rhai_trace` - Extract spans from Rhai scripts or ASTs and map Rhai errors to spans.

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
                BetterError::improve_eval_error(&e, code, &engine)
            );

            println!("POS: {:#?}", e.position());
        });
    }
}
