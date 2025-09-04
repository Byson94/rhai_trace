use ariadne::{Color, Label, Report, ReportKind, Source};
use rhai_trace::{BetterError};

fn display_error(code: &str, better: &BetterError) {
    let mut report = Report::build(
        ReportKind::Error,
        better.span.start()..better.span.end(), 
    )
    .with_message(&better.message)
    .with_label(
        Label::new(better.span.start()..better.span.end())
            .with_message(
                better.help
                    .as_deref()
                    .unwrap_or("See error details here"),
            )
            .with_color(Color::Red),
    );
    
    if let Some(note) = &better.note {
        report = report.with_label(
            Label::new(better.span.start()..better.span.end())
                .with_message(note)
                .with_color(Color::Cyan),
        );
    }

    report.finish().print(Source::from(code)).unwrap();
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code = r#"
to_json(
    1
);

return "test complete"
    "#;

    let engine = rhai::Engine::new();
    let mut scope = rhai::Scope::new();

    let result = engine.eval_with_scope::<rhai::Dynamic>(&mut scope, code);
    if let Err(err) = result {
        let better = rhai_trace::BetterError::improve_eval_error(&err, code, &engine)?;
        display_error(code, &better);
    }

    Ok(())
}
