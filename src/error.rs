use crate::span::Span;
use crate::tracer::SpanTracer;
use rhai::{Engine, EvalAltResult, ParseError, Position};
use std::error::Error;

/// Map a Rhai error to a Span or set of Spans.
#[derive(Debug, Clone)]
pub struct BetterError {
    pub message: String,
    pub help: Option<String>,
    pub hint: Option<String>,
    pub note: Option<String>,
    pub span: Span,
}

impl BetterError {
    /// Return a more informative Rhai evaluation error.
    pub fn improve_eval_error(
        error: &EvalAltResult,
        code: &str,
        engine: &Engine,
    ) -> Result<Self, Box<dyn Error>> {
        let pos = error.position();
        let line = pos.line().unwrap_or(0);
        let column = pos.position().unwrap_or(1);
        let help_hint = get_error_info(get_root_cause(error), error, engine, code);

        let span_tracer = SpanTracer::new();
        let spans = span_tracer.extract_from(code)?;
        let span = Self::find_span_for_position(&spans, line, column)
            .unwrap_or(Span::new(0, 0, line, column));

        Ok(BetterError {
            message: error.to_string(),
            help: if help_hint.help.is_empty() {
                None
            } else {
                Some(help_hint.help)
            },
            hint: if help_hint.hint.is_empty() {
                None
            } else {
                Some(help_hint.hint)
            },
            note: if help_hint.note.is_empty() {
                None
            } else {
                Some(help_hint.note)
            },
            span,
        })
    }

    /// Return a more informative Rhai parse error.
    pub fn improve_parse_error(error: &ParseError, code: &str) -> Result<Self, Box<dyn Error>> {
        let pos = error.position();
        let span = Span::from_pos(code, &pos);

        Ok(BetterError {
            message: error.to_string(),
            help: Some("Syntax error detected.".into()),
            hint: Some(
                "Check for missing tokens, unmatched parentheses, or invalid constructs.".into(),
            ),
            note: None,
            span,
        })
    }

    fn find_span_for_position(spans: &[Span], line: usize, column: usize) -> Option<Span> {
        if let Some(span) = spans.iter().find(|span| {
            span.line() == line
                && span.column() <= column
                && column <= span.column() + (span.end() - span.start())
        }) {
            return Some(span.clone());
        }

        spans.iter().find(|span| span.line() == line).cloned()
    }
}

fn get_deepest_position(error: &EvalAltResult) -> Position {
    match error {
        EvalAltResult::ErrorInFunctionCall(_, _, inner, _) => get_deepest_position(inner),
        EvalAltResult::ErrorInModule(_, inner, _) => get_deepest_position(inner),
        _ => error.position(),
    }
}

fn get_root_cause<'a>(err: &'a EvalAltResult) -> &'a EvalAltResult {
    match err {
        EvalAltResult::ErrorInFunctionCall(_, _, inner, _) => get_root_cause(inner),
        EvalAltResult::ErrorInModule(_, inner, _) => get_root_cause(inner),
        _ => err,
    }
}

fn get_error_info(
    root_err: &EvalAltResult,
    outer_err: &EvalAltResult,
    engine: &Engine,
    code: &str,
) -> ErrorHelp {
    let (help, hint) = match root_err {
        EvalAltResult::ErrorParsing(..) => (
            "Syntax error encountered while parsing.".into(),
            "Check for unmatched tokens, invalid constructs, or misplaced punctuation.".into(),
        ),
        EvalAltResult::ErrorVariableExists(name, ..) => (
            format!("Variable '{}' is already defined.", name),
            "Remove or rename the duplicate declaration.".into(),
        ),
        EvalAltResult::ErrorForbiddenVariable(name, ..) => (
            format!("Usage of forbidden variable '{}'.", name),
            "Avoid using reserved or protected variable names.".into(),
        ),
        EvalAltResult::ErrorVariableNotFound(name, ..) => (
            format!("Unknown variable '{}'.", name),
            "Check for typos or ensure the variable is initialized before use.".into(),
        ),
        EvalAltResult::ErrorPropertyNotFound(name, ..) => (
            format!("Property '{}' not found on this object.", name),
            "Verify the property name and the object’s available fields.".into(),
        ),
        EvalAltResult::ErrorFunctionNotFound(fn_sig, ..) => {
            let base = fn_sig.split('(').next().unwrap_or(fn_sig).trim();

            // Might be a bit less performant but I gotta pay the price of
            // having "kinda good" errors with Rhai.
            let ast = match engine.compile(code) {
                Ok(ast) => ast,
                Err(err) => {
                    return ErrorHelp {
                        help: format!("Failed to compile code for suggestions: {}", err),
                        hint: String::new(),
                        note: String::new(),
                    };
                }
            };

            let candidates: Vec<String> = ast
                .iter_functions()
                .filter(|f| f.name == base)
                .map(|f| {
                    let params = f.params.join(", ");
                    format!("{}({})", f.name, params)
                })
                .collect();

            if !candidates.is_empty() {
                (
                    format!("Function '{}' not found with this argument list.", fn_sig),
                    format!("Did you mean one of:\n  {}", candidates.join("\n    ")),
                )
            } else {
                (
                    format!("Function '{}' is not defined.", fn_sig),
                    "Check spelling, module path, or argument count.".into(),
                )
            }
        }
        EvalAltResult::ErrorModuleNotFound(name, ..) => (
            format!("Module '{}' could not be located.", name),
            "Check that the path is correct, the module is imported, and its code is valid.".into(),
        ),
        EvalAltResult::ErrorInFunctionCall(fn_name, msg, ..) => (
            format!("Error inside function '{}': {}", fn_name, msg),
            "Inspect the function implementation and arguments passed.".into(),
        ),
        EvalAltResult::ErrorInModule(name, ..) => (
            format!("Error while loading module '{}'.", name),
            "Check the module code for syntax or runtime errors.".into(),
        ),
        EvalAltResult::ErrorUnboundThis(..) => (
            "`this` is unbound in this context.".into(),
            "Only use `this` inside methods or bound closures.".into(),
        ),
        EvalAltResult::ErrorMismatchDataType(found, expected, ..) => (
            format!(
                "Data type mismatch: found '{}', expected '{}'.",
                found, expected
            ),
            "Convert or cast values to the required type.".into(),
        ),
        EvalAltResult::ErrorMismatchOutputType(found, expected, ..) => (
            format!(
                "Return type mismatch: found '{}', expected '{}'.",
                found, expected
            ),
            "Ensure your function returns the correct type.".into(),
        ),
        EvalAltResult::ErrorIndexingType(typ, ..) => (
            format!("Cannot index into value of type '{}'.", typ),
            "Only arrays, maps, bitfields, or strings support indexing.".into(),
        ),
        EvalAltResult::ErrorArrayBounds(len, idx, ..) => (
            format!("Array index {} out of bounds (0..{}).", idx, len),
            "Use a valid index within the array’s range.".into(),
        ),
        EvalAltResult::ErrorStringBounds(len, idx, ..) => (
            format!("String index {} out of bounds (0..{}).", idx, len),
            "Ensure you index only valid character positions.".into(),
        ),
        EvalAltResult::ErrorBitFieldBounds(len, idx, ..) => (
            format!("Bitfield index {} out of bounds (0..{}).", idx, len),
            "Use a valid bit position within the bitfield’s size.".into(),
        ),
        EvalAltResult::ErrorFor(..) => (
            "`for` loop value is not iterable.".into(),
            "Iterate only over arrays, strings, ranges, or iterators.".into(),
        ),
        EvalAltResult::ErrorDataRace(name, ..) => (
            format!("Data race detected on '{}'.", name),
            "Avoid shared mutable data or use synchronization primitives.".into(),
        ),
        EvalAltResult::ErrorAssignmentToConstant(name, ..) => (
            format!("Cannot assign to constant '{}'.", name),
            "Constants cannot be reassigned after declaration.".into(),
        ),
        EvalAltResult::ErrorDotExpr(field, ..) => (
            format!("Invalid member access '{}'.", field),
            "Verify the object has this member or method.".into(),
        ),
        EvalAltResult::ErrorArithmetic(msg, ..) => {
            ("Arithmetic error encountered.".into(), msg.clone())
        }
        EvalAltResult::ErrorTooManyOperations(..) => (
            "Script exceeded the maximum number of operations.".into(),
            "Break complex expressions into smaller steps or increase the limit.".into(),
        ),
        EvalAltResult::ErrorTooManyModules(..) => (
            "Too many modules have been loaded.".into(),
            "Use fewer modules or increase the module limit.".into(),
        ),
        EvalAltResult::ErrorStackOverflow(..) => (
            "Call stack overflow detected.".into(),
            "Check for infinite recursion or deeply nested calls.".into(),
        ),
        EvalAltResult::ErrorDataTooLarge(name, ..) => (
            format!("Data '{}' is too large to handle.", name),
            "Use smaller data sizes or adjust engine limits.".into(),
        ),
        EvalAltResult::ErrorTerminated(..) => (
            "Script execution was terminated.".into(),
            "This occurs when a `stop` or external termination is triggered.".into(),
        ),
        EvalAltResult::ErrorCustomSyntax(msg, options, ..) => (
            format!("Custom syntax error: {}.", msg),
            format!("Expected one of: {}.", options.join(", ")),
        ),
        EvalAltResult::ErrorRuntime(..) => (
            "Runtime error encountered.".into(),
            "Inspect the error message and script logic for issues.".into(),
        ),
        EvalAltResult::LoopBreak(..) => (
            "`break` used outside of a loop.".into(),
            "Only use `break` inside `for` or `while` loops.".into(),
        ),
        EvalAltResult::Return(..) => (
            "`return` statement encountered.".into(),
            "Script terminated with an explicit return value.".into(),
        ),
        _ => (
            "Unknown error".into(),
            "No additional information available for this error.".into(),
        ),
    };

    let note = match outer_err {
        EvalAltResult::ErrorInFunctionCall(fn_name, ..) => format!(
            "This error occurred during a call to '{}'. Inspecting the function implementation and arguments passed may help solve this error.",
            fn_name
        ),

        EvalAltResult::ErrorInModule(mod_name, ..) => format!(
            "This happened while loading the module '{}'. Tip: Check the module code for syntax or runtime errors",
            mod_name
        ),

        EvalAltResult::ErrorRuntime(..) => {
            "A runtime error bubbled up from a lower-level operation.".into()
        }

        _ => "".into(),
    };

    ErrorHelp { help, hint, note }
}

struct ErrorHelp {
    help: String,
    hint: String,
    note: String,
}
